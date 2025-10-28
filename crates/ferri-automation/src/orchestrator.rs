//! Job orchestrator for ferri-flow execution
//!
//! This module handles:
//! - Parallel job execution
//! - Dependency resolution via topological sort
//! - Context management and expression evaluation
//! - Real-time status updates

use crate::executors::ExecutorRegistry;
use crate::expressions::EvaluationContext;
use crate::flow::{FlowDocument, Job, JobStatus, JobUpdate, Update};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

/// Orchestrator state for flow execution
pub struct FlowOrchestrator {
    flow: FlowDocument,
    base_path: std::path::PathBuf,
    runtime_inputs: HashMap<String, String>,
    executor_registry: Arc<ExecutorRegistry>,
}

impl FlowOrchestrator {
    pub fn new(
        flow: FlowDocument,
        base_path: &Path,
        runtime_inputs: HashMap<String, String>,
    ) -> Self {
        Self {
            flow,
            base_path: base_path.to_path_buf(),
            runtime_inputs,
            executor_registry: Arc::new(ExecutorRegistry::new()),
        }
    }

    /// Execute the entire flow
    pub fn execute(&self) -> io::Result<PathBuf> {
        // Create a unique run ID
        let run_id = format!(
            "{}-{}",
            self.flow.metadata.name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        // Create the runs directory
        let runs_dir = self.base_path.join(".ferri").join("runs");
        fs::create_dir_all(&runs_dir)?;
        let log_path = runs_dir.join(format!("{}.log", run_id));
        let log_file = fs::File::create(&log_path)?;
        let writer = Arc::new(Mutex::new(io::BufWriter::new(log_file)));

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
            self.execute_wave(
                &wave,
                Arc::clone(&job_outputs),
                &workspace_paths,
                Arc::clone(&writer),
            )?;
        }

        Ok(log_path)
    }

    /// Build a mapping of workspace names to their absolute paths
    fn build_workspace_paths(&self, workspace_root: &Path) -> HashMap<String, PathBuf> {
        let mut paths = HashMap::new();
        if let Some(workspaces) = &self.flow.spec.workspaces {
            for workspace in workspaces {
                paths.insert(workspace.name.clone(), workspace_root.join(&workspace.name));
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
                "Circular dependency detected in job graph",
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
        writer: Arc<Mutex<io::BufWriter<fs::File>>>,
    ) -> io::Result<()> {
        // Send Pending status for all jobs in this wave
        for job_id in wave {
            let update = Update::Job(JobUpdate {
                job_id: job_id.clone(),
                status: JobStatus::Pending,
            });
            let mut writer_lock = writer.lock().unwrap();
            writeln!(writer_lock, "{}", serde_json::to_string(&update).unwrap())?;
            writer_lock.flush()?;
        }

        let mut handles = Vec::new();

        for job_id in wave {
            let job = self.flow.spec.jobs.get(job_id).unwrap().clone();
            let job_id = job_id.clone();
            let base_path = self.base_path.clone();
            let runtime_inputs = self.runtime_inputs.clone();
            let job_outputs_clone = Arc::clone(&job_outputs);
            let flow = self.flow.clone();

            let workspace_paths_clone = workspace_paths.clone();

            let registry_clone = Arc::clone(&self.executor_registry);
            let writer_clone = Arc::clone(&writer);

            let handle = thread::spawn(move || {
                Self::execute_job(
                    &job_id,
                    &job,
                    &base_path,
                    &runtime_inputs,
                    job_outputs_clone,
                    &flow,
                    &workspace_paths_clone,
                    registry_clone,
                    writer_clone,
                )
            });

            handles.push(handle);
        }

        // Wait for all jobs in this wave to complete
        let mut errors = Vec::new();
        for (idx, handle) in handles.into_iter().enumerate() {
            let job_id = &wave[idx];
            let (status, error_msg) = match handle.join() {
                Ok(Ok(())) => (JobStatus::Succeeded, None),
                Ok(Err(e)) => {
                    let msg = e.to_string();
                    (
                        JobStatus::Failed(msg.clone()),
                        Some(format!("Job '{}' failed: {}", job_id, msg)),
                    )
                }
                Err(e) => {
                    let msg = format!("Job thread panicked: {:?}", e);
                    (JobStatus::Failed(msg.clone()), Some(msg))
                }
            };

            let update = Update::Job(JobUpdate {
                job_id: job_id.clone(),
                status,
            });
            let mut writer_lock = writer.lock().unwrap();
            writeln!(writer_lock, "{}", serde_json::to_string(&update).unwrap())?;
            writer_lock.flush()?;

            if let Some(msg) = error_msg {
                errors.push(msg);
            }
        }

        if !errors.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, errors.join("; ")));
        }

        Ok(())
    }

    /// Execute a single job
    fn execute_job(
        job_id: &str,
        job: &Job,
        base_path: &Path,
        runtime_inputs: &HashMap<String, String>,
        job_outputs: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
        _flow: &FlowDocument,
        _workspace_paths: &HashMap<String, PathBuf>,
        executor_registry: Arc<ExecutorRegistry>,
        writer: Arc<Mutex<io::BufWriter<fs::File>>>,
    ) -> io::Result<()> {
        // Run the job using the appropriate executor
        let handle =
            Self::run_job_executor(job_id, job, base_path, executor_registry, writer)?;
        handle.0.join().unwrap()?; // Wait for the executor to finish

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

    /// Selects and runs the appropriate executor for a job
    fn run_job_executor(
        job_id: &str,
        job: &Job,
        base_path: &Path,
        executor_registry: Arc<ExecutorRegistry>,
        writer: Arc<Mutex<io::BufWriter<fs::File>>>,
    ) -> io::Result<crate::executors::ExecutionHandle> {
        // --- Executor Selection ---
        let executor_name = job.runs_on.as_deref().unwrap_or("process");
        let executor = executor_registry.get(executor_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Executor '{}' not found for job '{}'",
                    executor_name, job_id
                ),
            )
        })?;

        // Secrets are not yet implemented, so we pass an empty HashMap.
        let handle = executor.execute(job_id, job, base_path, &HashMap::new(), writer)?;

        Ok(handle)
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
    use crate::flow::{FlowSpec, Metadata, Workspace};

    fn create_test_flow(
        jobs: HashMap<String, Job>,
        workspaces: Option<Vec<Workspace>>,
    ) -> FlowDocument {
        FlowDocument {
            api_version: "ferri.flow/v1alpha1".to_string(),
            kind: "Flow".to_string(),
            metadata: Metadata {
                name: "test-flow".to_string(),
                labels: None,
                annotations: None,
            },
            spec: FlowSpec {
                inputs: None,
                workspaces,
                jobs,
            },
        }
    }

    #[test]
    fn test_simple_dependency_resolution() {
        let mut jobs = HashMap::new();
        jobs.insert(
            "job1".to_string(),
            Job {
                name: Some("Job 1".to_string()),
                runs_on: Some("ubuntu-latest".to_string()),
                needs: None,
                steps: vec![],
            },
        );
        jobs.insert(
            "job2".to_string(),
            Job {
                name: Some("Job 2".to_string()),
                runs_on: Some("ubuntu-latest".to_string()),
                needs: Some(vec!["job1".to_string()]),
                steps: vec![],
            },
        );
        jobs.insert(
            "job3".to_string(),
            Job {
                name: Some("Job 3".to_string()),
                runs_on: Some("ubuntu-latest".to_string()),
                needs: Some(vec!["job2".to_string()]),
                steps: vec![],
            },
        );

        let flow = create_test_flow(jobs, None);
        let orchestrator = FlowOrchestrator::new(flow, Path::new("/tmp"), HashMap::new());
        let order = orchestrator.resolve_execution_order().unwrap();

        assert_eq!(order.len(), 3);
        assert_eq!(order[0], vec!["job1"]);
        assert_eq!(order[1], vec!["job2"]);
        assert_eq!(order[2], vec!["job3"]);
    }

    #[test]
    fn test_parallel_jobs() {
        let mut jobs = HashMap::new();
        jobs.insert(
            "job1".to_string(),
            Job {
                name: Some("Job 1".to_string()),
                runs_on: Some("ubuntu-latest".to_string()),
                needs: None,
                steps: vec![],
            },
        );
        jobs.insert(
            "job2".to_string(),
            Job {
                name: Some("Job 2".to_string()),
                runs_on: Some("ubuntu-latest".to_string()),
                needs: None,
                steps: vec![],
            },
        );
        jobs.insert(
            "job3".to_string(),
            Job {
                name: Some("Job 3".to_string()),
                runs_on: Some("ubuntu-latest".to_string()),
                needs: Some(vec!["job1".to_string(), "job2".to_string()]),
                steps: vec![],
            },
        );

        let flow = create_test_flow(jobs, None);
        let orchestrator = FlowOrchestrator::new(flow, Path::new("/tmp"), HashMap::new());
        let order = orchestrator.resolve_execution_order().unwrap();

        assert_eq!(order.len(), 2);
        assert_eq!(order[0].len(), 2);
        assert!(order[0].contains(&"job1".to_string()));
        assert!(order[0].contains(&"job2".to_string()));
        assert_eq!(order[1], vec!["job3"]);
    }

    #[test]
    fn test_job_state_tracking() {
        let mut jobs = HashMap::new();
        jobs.insert(
            "job1".to_string(),
            Job {
                name: Some("Job 1".to_string()),
                runs_on: Some("ubuntu-latest".to_string()),
                needs: None,
                steps: vec![],
            },
        );
        jobs.insert(
            "job2".to_string(),
            Job {
                name: Some("Job 2".to_string()),
                runs_on: Some("ubuntu-latest".to_string()),
                needs: Some(vec!["job1".to_string()]),
                steps: vec![],
            },
        );

        let flow = create_test_flow(jobs, None);
        let orchestrator = FlowOrchestrator::new(flow, Path::new("/tmp"), HashMap::new());
        let order = orchestrator.resolve_execution_order().unwrap();

        assert_eq!(order.len(), 2);
        assert_eq!(order[0], vec!["job1"]);
        assert_eq!(order[1], vec!["job2"]);
    }

    #[test]
    fn test_workspace_creation_and_cleanup() {
        let mut jobs = HashMap::new();
        jobs.insert(
            "job1".to_string(),
            Job {
                name: Some("Test Job".to_string()),
                runs_on: Some("ubuntu-latest".to_string()),
                needs: None,
                steps: vec![],
            },
        );
        let workspaces = Some(vec![
            Workspace {
                name: "data".to_string(),
            },
            Workspace {
                name: "logs".to_string(),
            },
        ]);

        let flow = create_test_flow(jobs, workspaces);
        let orchestrator = FlowOrchestrator::new(flow, Path::new("/tmp"), HashMap::new());
        let workspace_root = orchestrator.create_workspace_directories().unwrap();

        assert!(workspace_root.exists() && workspace_root.is_dir());
        assert!(workspace_root.join("data").exists());
        assert!(workspace_root.join("logs").exists());

        drop(WorkspaceCleanupGuard::new(workspace_root.clone()));
        assert!(!workspace_root.exists());
    }

    #[test]
    fn test_workspace_paths_mapping() {
        let mut jobs = HashMap::new();
        jobs.insert(
            "job1".to_string(),
            Job {
                name: Some("Test Job".to_string()),
                runs_on: Some("ubuntu-latest".to_string()),
                needs: None,
                steps: vec![],
            },
        );
        let workspaces = Some(vec![
            Workspace {
                name: "data".to_string(),
            },
            Workspace {
                name: "cache".to_string(),
            },
        ]);

        let flow = create_test_flow(jobs, workspaces);
        let orchestrator = FlowOrchestrator::new(flow, Path::new("/tmp"), HashMap::new());
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
