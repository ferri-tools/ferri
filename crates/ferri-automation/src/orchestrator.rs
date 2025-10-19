//! Job orchestrator for ferri-flow execution
//!
//! This module handles:
//! - Parallel job execution
//! - Dependency resolution via topological sort
//! - Context management and expression evaluation
//! - Real-time status updates

use crate::executors::ExecutorRegistry;
use crate::expressions::EvaluationContext;
use crate::flow::{FlowDocument, Job, Update, JobUpdate, JobStatus};
use crossbeam_channel::Sender;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

/// Orchestrator state for flow execution
pub struct FlowOrchestrator {
    flow: FlowDocument,
    base_path: std::path::PathBuf,
    update_sender: Sender<Update>,
    runtime_inputs: HashMap<String, String>,
    executor_registry: Arc<ExecutorRegistry>,
}

impl FlowOrchestrator {
    pub fn new(
        flow: FlowDocument,
        base_path: &Path,
        update_sender: Sender<Update>,
        runtime_inputs: HashMap<String, String>,
    ) -> Self {
        Self {
            flow,
            base_path: base_path.to_path_buf(),
            update_sender,
            runtime_inputs,
            executor_registry: Arc::new(ExecutorRegistry::new()),
        }
    }

    /// Execute the entire flow
    pub fn execute(&self) -> io::Result<()> {
        // Create temporary workspace directories
        let workspace_root = self.create_workspace_directories()?;

        // Build workspace path mapping
        let workspace_paths = self.build_workspace_paths(&workspace_root);

        // Ensure cleanup happens even if execution fails
        let _cleanup_guard = WorkspaceCleanupGuard::new(workspace_root.clone());

        // Resolve execution order
        let execution_order = self.resolve_execution_order()?;

        // Shared state for job outputs
        let job_outputs = Arc::new(Mutex::new(HashMap::<String, HashMap<String, String>>::new()));

        // Execute jobs in waves (each wave contains jobs that can run in parallel)
        for wave in execution_order {
            self.execute_wave(&wave, Arc::clone(&job_outputs), &workspace_paths)?;
        }

        Ok(())
    }

    /// Build a mapping of workspace names to their absolute paths
    fn build_workspace_paths(&self, workspace_root: &Path) -> HashMap<String, PathBuf> {
        let mut paths = HashMap::new();
        if let Some(workspaces) = &self.flow.spec.workspaces {
            for workspace in workspaces {
                paths.insert(
                    workspace.name.clone(),
                    workspace_root.join(&workspace.name),
                );
            }
        }
        paths
    }

    /// Create temporary workspace directories for this flow run
    fn create_workspace_directories(&self) -> io::Result<PathBuf> {
        // Create a unique temporary directory for this flow run
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let flow_name = &self.flow.metadata.name;
        let workspace_root = std::env::temp_dir()
            .join("ferri-workspaces")
            .join(format!("{}-{}", flow_name, timestamp));

        // Create the root directory
        fs::create_dir_all(&workspace_root)?;

        // Create a subdirectory for each workspace defined in the flow
        if let Some(workspaces) = &self.flow.spec.workspaces {
            for workspace in workspaces {
                let workspace_dir = workspace_root.join(&workspace.name);
                fs::create_dir_all(&workspace_dir)?;
            }
        }

        Ok(workspace_root)
    }

    /// Resolve job execution order using topological sort
    fn resolve_execution_order(&self) -> io::Result<Vec<Vec<String>>> {
        let jobs = &self.flow.spec.jobs;
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize graph
        for job_id in jobs.keys() {
            in_degree.insert(job_id.clone(), 0);
            graph.insert(job_id.clone(), Vec::new());
        }

        // Build dependency graph
        for (job_id, job) in jobs {
            if let Some(needs) = &job.needs {
                for dep in needs {
                    graph.get_mut(dep).unwrap().push(job_id.clone());
                    *in_degree.get_mut(job_id).unwrap() += 1;
                }
            }
        }

        // Topological sort with wave tracking
        let mut waves = Vec::new();
        let mut queue = VecDeque::new();

        // Start with jobs that have no dependencies
        for (job_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(job_id.clone());
            }
        }

        let mut processed = 0;
        while !queue.is_empty() {
            // Current wave: all jobs in the queue now
            let mut current_wave = Vec::new();
            let wave_size = queue.len();

            for _ in 0..wave_size {
                if let Some(job_id) = queue.pop_front() {
                    current_wave.push(job_id.clone());
                    processed += 1;

                    // Reduce in-degree for dependent jobs
                    if let Some(dependents) = graph.get(&job_id) {
                        for dep_job in dependents {
                            let degree = in_degree.get_mut(dep_job).unwrap();
                            *degree -= 1;
                            if *degree == 0 {
                                queue.push_back(dep_job.clone());
                            }
                        }
                    }
                }
            }

            if !current_wave.is_empty() {
                waves.push(current_wave);
            }
        }

        // Check for cycles
        if processed != jobs.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Circular dependency detected in job graph"
            ));
        }

        Ok(waves)
    }

    /// Execute a wave of jobs in parallel
    fn execute_wave(
        &self,
        wave: &[String],
        job_outputs: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
        workspace_paths: &HashMap<String, PathBuf>,
    ) -> io::Result<()> {
        // Send Pending status for all jobs in this wave
        for job_id in wave {
            self.update_sender.send(Update::Job(JobUpdate {
                job_id: job_id.clone(),
                status: JobStatus::Pending,
            })).unwrap();
        }

        let mut handles = Vec::new();

        for job_id in wave {
            let job = self.flow.spec.jobs.get(job_id).unwrap().clone();
            let job_id = job_id.clone();
            let base_path = self.base_path.clone();
            let update_sender = self.update_sender.clone();
            let runtime_inputs = self.runtime_inputs.clone();
            let job_outputs_clone = Arc::clone(&job_outputs);
            let flow = self.flow.clone();

            let workspace_paths_clone = workspace_paths.clone();

            let registry_clone = Arc::clone(&self.executor_registry);

            let handle = thread::spawn(move || {
                Self::execute_job(
                    &job_id,
                    &job,
                    &base_path,
                    update_sender,
                    &runtime_inputs,
                    job_outputs_clone,
                    &flow,
                    &workspace_paths_clone,
                    registry_clone,
                )
            });

            handles.push(handle);
        }

        // Wait for all jobs in this wave to complete
        let mut errors = Vec::new();
        for (idx, handle) in handles.into_iter().enumerate() {
            let job_id = &wave[idx];
            match handle.join() {
                Ok(Ok(())) => {
                    // Job succeeded
                    self.update_sender.send(Update::Job(JobUpdate {
                        job_id: job_id.clone(),
                        status: JobStatus::Succeeded,
                    })).unwrap();
                }
                Ok(Err(e)) => {
                    // Job returned an error
                    let error_msg = e.to_string();
                    self.update_sender.send(Update::Job(JobUpdate {
                        job_id: job_id.clone(),
                        status: JobStatus::Failed(error_msg.clone()),
                    })).unwrap();
                    errors.push(format!("Job '{}' failed: {}", job_id, error_msg));
                }
                Err(e) => {
                    // Thread panicked
                    let panic_msg = format!("Job thread panicked: {:?}", e);
                    self.update_sender.send(Update::Job(JobUpdate {
                        job_id: job_id.clone(),
                        status: JobStatus::Failed(panic_msg.clone()),
                    })).unwrap();
                    errors.push(panic_msg);
                }
            }
        }

        if !errors.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                errors.join("; ")
            ));
        }

        Ok(())
    }

    /// Execute a single job
    fn execute_job(
        job_id: &str,
        job: &Job,
        base_path: &Path,
        update_sender: Sender<Update>,
        runtime_inputs: &HashMap<String, String>,
        job_outputs: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
        _flow: &FlowDocument,
        _workspace_paths: &HashMap<String, PathBuf>,
        executor_registry: Arc<ExecutorRegistry>,
    ) -> io::Result<()> {
        // Send Running status
        update_sender.send(Update::Job(JobUpdate {
            job_id: job_id.to_string(),
            status: JobStatus::Running,
        })).unwrap();

        // --- Executor Selection ---
        let executor_name = job.runs_on.as_deref().unwrap_or("process");
        let executor = executor_registry.get(executor_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Executor '{}' not found for job '{}'", executor_name, job_id),
            )
        })?;

        let handle = executor.execute(job_id, job, base_path, &HashMap::new(), update_sender.clone())?;

        // Wait for the executor to finish
        match handle.0.join() {
            Ok(Ok(())) => {
                // Job succeeded
            }
            Ok(Err(e)) => {
                // Job returned an error
                return Err(e);
            }
            Err(e) => {
                // Thread panicked
                let panic_msg = format!("Job thread panicked: {:?}", e);
                return Err(io::Error::new(io::ErrorKind::Other, panic_msg));
            }
        }

        // Build evaluation context
        let mut ctx = EvaluationContext::new().with_inputs(runtime_inputs.clone());

        // Add job outputs from dependencies
        {
            let outputs = job_outputs.lock().unwrap();
            for (dep_job_id, dep_outputs) in outputs.iter() {
                for (output_name, output_value) in dep_outputs.iter() {
                    ctx.add_job_output(
                        dep_job_id.clone(),
                        output_name.clone(),
                        output_value.clone(),
                    );
                }
            }
        }

        // TODO: Collect job-level outputs and store them in job_outputs

        Ok(())
    }
}

/// Guard that ensures workspace directories are cleaned up when dropped
struct WorkspaceCleanupGuard {
    workspace_root: PathBuf,
}

impl WorkspaceCleanupGuard {
    fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl Drop for WorkspaceCleanupGuard {
    fn drop(&mut self) {
        // Attempt to remove the workspace directory
        // Ignore errors during cleanup to avoid panicking in drop
        let _ = fs::remove_dir_all(&self.workspace_root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow::{FlowSpec, Metadata};

    #[test]
    fn test_simple_dependency_resolution() {
        // Create a simple flow: job1 -> job2 -> job3
        let mut jobs = HashMap::new();

        jobs.insert("job1".to_string(), Job {
            name: Some("Job 1".to_string()),
            runs_on: "ubuntu-latest".to_string(),
            needs: None,
            steps: vec![],
        });

        jobs.insert("job2".to_string(), Job {
            name: Some("Job 2".to_string()),
            runs_on: "ubuntu-latest".to_string(),
            needs: Some(vec!["job1".to_string()]),
            steps: vec![],
        });

        jobs.insert("job3".to_string(), Job {
            name: Some("Job 3".to_string()),
            runs_on: "ubuntu-latest".to_string(),
            needs: Some(vec!["job2".to_string()]),
            steps: vec![],
        });

        let flow = FlowDocument {
            api_version: "ferri.flow/v1alpha1".to_string(),
            kind: "Flow".to_string(),
            metadata: Metadata {
                name: "test-flow".to_string(),
                labels: None,
                annotations: None,
            },
            spec: FlowSpec {
                inputs: None,
                workspaces: None,
                jobs,
            },
        };

        let (tx, _rx) = crossbeam_channel::unbounded();
        let orchestrator = FlowOrchestrator::new(
            flow,
            Path::new("/tmp"),
            tx,
            HashMap::new(),
        );

        let order = orchestrator.resolve_execution_order().unwrap();

        assert_eq!(order.len(), 3);
        assert_eq!(order[0], vec!["job1"]);
        assert_eq!(order[1], vec!["job2"]);
        assert_eq!(order[2], vec!["job3"]);
    }

    #[test]
    fn test_parallel_jobs() {
        // Create jobs that can run in parallel
        let mut jobs = HashMap::new();

        jobs.insert("job1".to_string(), Job {
            name: Some("Job 1".to_string()),
            runs_on: "ubuntu-latest".to_string(),
            needs: None,
            steps: vec![],
        });

        jobs.insert("job2".to_string(), Job {
            name: Some("Job 2".to_string()),
            runs_on: "ubuntu-latest".to_string(),
            needs: None,
            steps: vec![],
        });

        jobs.insert("job3".to_string(), Job {
            name: Some("Job 3".to_string()),
            runs_on: "ubuntu-latest".to_string(),
            needs: Some(vec!["job1".to_string(), "job2".to_string()]),
            steps: vec![],
        });

        let flow = FlowDocument {
            api_version: "ferri.flow/v1alpha1".to_string(),
            kind: "Flow".to_string(),
            metadata: Metadata {
                name: "test-flow".to_string(),
                labels: None,
                annotations: None,
            },
            spec: FlowSpec {
                inputs: None,
                workspaces: None,
                jobs,
            },
        };

        let (tx, _rx) = crossbeam_channel::unbounded();
        let orchestrator = FlowOrchestrator::new(
            flow,
            Path::new("/tmp"),
            tx,
            HashMap::new(),
        );

        let order = orchestrator.resolve_execution_order().unwrap();

        assert_eq!(order.len(), 2);
        assert_eq!(order[0].len(), 2); // job1 and job2 in parallel
        assert!(order[0].contains(&"job1".to_string()));
        assert!(order[0].contains(&"job2".to_string()));
        assert_eq!(order[1], vec!["job3"]);
    }

    #[test]
    fn test_job_state_tracking() {
        // Create a simple flow with two jobs
        let mut jobs = HashMap::new();

        jobs.insert("job1".to_string(), Job {
            name: Some("Job 1".to_string()),
            runs_on: "ubuntu-latest".to_string(),
            needs: None,
            steps: vec![],
        });

        jobs.insert("job2".to_string(), Job {
            name: Some("Job 2".to_string()),
            runs_on: "ubuntu-latest".to_string(),
            needs: Some(vec!["job1".to_string()]),
            steps: vec![],
        });

        let flow = FlowDocument {
            api_version: "ferri.flow/v1alpha1".to_string(),
            kind: "Flow".to_string(),
            metadata: Metadata {
                name: "test-flow".to_string(),
                labels: None,
                annotations: None,
            },
            spec: FlowSpec {
                inputs: None,
                workspaces: None,
                jobs,
            },
        };

        let (tx, rx) = crossbeam_channel::unbounded();
        let orchestrator = FlowOrchestrator::new(
            flow,
            Path::new("/tmp"),
            tx,
            HashMap::new(),
        );

        // Get execution order
        let order = orchestrator.resolve_execution_order().unwrap();
        assert_eq!(order.len(), 2);

        // Verify we have the channel set up correctly
        // (Full execution test would require mocking the job system)
        // For now, just verify the structure is correct
        assert_eq!(order[0], vec!["job1"]);
        assert_eq!(order[1], vec!["job2"]);

        // The channel rx would receive job status updates during actual execution:
        // - Update::Job(JobUpdate { job_id: "job1", status: Pending })
        // - Update::Job(JobUpdate { job_id: "job1", status: Running })
        // - Update::Job(JobUpdate { job_id: "job1", status: Succeeded })
        // - Update::Job(JobUpdate { job_id: "job2", status: Pending })
        // - Update::Job(JobUpdate { job_id: "job2", status: Running })
        // - Update::Job(JobUpdate { job_id: "job2", status: Succeeded })
        drop(rx); // Verify channel was created
    }

    #[test]
    fn test_workspace_creation_and_cleanup() {
        use crate::flow::Workspace;

        // Create a flow with workspace definitions
        let mut jobs = HashMap::new();
        jobs.insert("job1".to_string(), Job {
            name: Some("Test Job".to_string()),
            runs_on: "ubuntu-latest".to_string(),
            needs: None,
            steps: vec![],
        });

        let flow = FlowDocument {
            api_version: "ferri.flow/v1alpha1".to_string(),
            kind: "Flow".to_string(),
            metadata: Metadata {
                name: "test-workspace-flow".to_string(),
                labels: None,
                annotations: None,
            },
            spec: FlowSpec {
                inputs: None,
                workspaces: Some(vec![
                    Workspace { name: "data".to_string() },
                    Workspace { name: "logs".to_string() },
                ]),
                jobs,
            },
        };

        let (tx, _rx) = crossbeam_channel::unbounded();
        let orchestrator = FlowOrchestrator::new(
            flow,
            Path::new("/tmp"),
            tx,
            HashMap::new(),
        );

        // Create workspaces
        let workspace_root = orchestrator.create_workspace_directories().unwrap();

        // Verify root directory was created
        assert!(workspace_root.exists());
        assert!(workspace_root.is_dir());

        // Verify workspace subdirectories were created
        let data_workspace = workspace_root.join("data");
        let logs_workspace = workspace_root.join("logs");
        assert!(data_workspace.exists());
        assert!(data_workspace.is_dir());
        assert!(logs_workspace.exists());
        assert!(logs_workspace.is_dir());

        // Test cleanup guard
        {
            let _guard = WorkspaceCleanupGuard::new(workspace_root.clone());
            // Guard goes out of scope here
        }

        // Verify directory was cleaned up
        assert!(!workspace_root.exists());
    }

    #[test]
    fn test_workspace_paths_mapping() {
        use crate::flow::Workspace;
        use std::path::PathBuf;

        // Create a flow with workspaces
        let mut jobs = HashMap::new();
        jobs.insert("job1".to_string(), Job {
            name: Some("Test Job".to_string()),
            runs_on: "ubuntu-latest".to_string(),
            needs: None,
            steps: vec![],
        });

        let flow = FlowDocument {
            api_version: "ferri.flow/v1alpha1".to_string(),
            kind: "Flow".to_string(),
            metadata: Metadata {
                name: "test-flow".to_string(),
                labels: None,
                annotations: None,
            },
            spec: FlowSpec {
                inputs: None,
                workspaces: Some(vec![
                    Workspace { name: "data".to_string() },
                    Workspace { name: "cache".to_string() },
                ]),
                jobs,
            },
        };

        let (tx, _rx) = crossbeam_channel::unbounded();
        let orchestrator = FlowOrchestrator::new(
            flow,
            Path::new("/tmp"),
            tx,
            HashMap::new(),
        );

        // Test workspace path building
        let workspace_root = PathBuf::from("/tmp/test-workspaces");
        let workspace_paths = orchestrator.build_workspace_paths(&workspace_root);

        assert_eq!(workspace_paths.len(), 2);
        assert_eq!(
            workspace_paths.get("data").unwrap(),
            &workspace_root.join("data")
        );
        assert_eq!(
            workspace_paths.get("cache").unwrap(),
            &workspace_root.join("cache")
        );
    }
}
