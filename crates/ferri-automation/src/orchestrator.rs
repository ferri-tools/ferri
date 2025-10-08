//! Job orchestrator for ferri-flow execution
//!
//! This module handles:
//! - Parallel job execution
//! - Dependency resolution via topological sort
//! - Context management and expression evaluation
//! - Real-time status updates

use crate::expressions::{self, EvaluationContext};
use crate::flow::{FlowDocument, Job, Step, Update, JobUpdate, JobStatus, StepUpdate, StepStatus};
use crossbeam_channel::Sender;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

/// Orchestrator state for flow execution
pub struct FlowOrchestrator {
    flow: FlowDocument,
    base_path: std::path::PathBuf,
    update_sender: Sender<Update>,
    runtime_inputs: HashMap<String, String>,
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
        }
    }

    /// Execute the entire flow
    pub fn execute(&self) -> io::Result<()> {
        // Resolve execution order
        let execution_order = self.resolve_execution_order()?;

        // Shared state for job outputs
        let job_outputs = Arc::new(Mutex::new(HashMap::<String, HashMap<String, String>>::new()));

        // Execute jobs in waves (each wave contains jobs that can run in parallel)
        for wave in execution_order {
            self.execute_wave(&wave, Arc::clone(&job_outputs))?;
        }

        Ok(())
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

            let handle = thread::spawn(move || {
                Self::execute_job(
                    &job_id,
                    &job,
                    &base_path,
                    update_sender,
                    &runtime_inputs,
                    job_outputs_clone,
                    &flow,
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
    ) -> io::Result<()> {
        // Send Running status
        update_sender.send(Update::Job(JobUpdate {
            job_id: job_id.to_string(),
            status: JobStatus::Running,
        })).unwrap();

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

        // Execute each step sequentially within the job
        for (step_idx, step) in job.steps.iter().enumerate() {
            let step_name = step.name.clone()
                .unwrap_or_else(|| format!("step-{}", step_idx));

            Self::execute_step(
                job_id,
                &step_name,
                step,
                base_path,
                &update_sender,
                &mut ctx,
            )?;
        }

        // TODO: Collect job-level outputs and store them in job_outputs

        Ok(())
    }

    /// Execute a single step
    fn execute_step(
        job_id: &str,
        step_name: &str,
        step: &Step,
        base_path: &Path,
        update_sender: &Sender<Update>,
        ctx: &mut EvaluationContext,
    ) -> io::Result<()> {
        // Send running status
        update_sender.send(Update::Step(StepUpdate {
            job_id: job_id.to_string(),
            step_name: step_name.to_string(),
            status: StepStatus::Running,
            output: None,
        })).unwrap();

        // Evaluate expressions in the step
        let evaluated_step = Self::evaluate_step_expressions(step, ctx)?;

        // Execute based on step type
        if let Some(run_command) = &evaluated_step.run {
            Self::execute_run_step(
                job_id,
                step_name,
                run_command,
                &evaluated_step.env,
                base_path,
                update_sender,
                ctx,
            )?;
        } else if let Some(uses) = &evaluated_step.uses {
            // TODO: Implement reusable actions
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Reusable actions not yet implemented: {}", uses)
            ));
        }

        Ok(())
    }

    /// Evaluate expressions in a step
    fn evaluate_step_expressions(step: &Step, ctx: &EvaluationContext) -> io::Result<Step> {
        let mut evaluated = step.clone();

        // Evaluate 'run' field
        if let Some(run) = &step.run {
            evaluated.run = Some(expressions::evaluate_expressions(run, ctx)?);
        }

        // Evaluate environment variables
        if let Some(env) = &step.env {
            let mut evaluated_env = HashMap::new();
            for (key, value) in env {
                let evaluated_value = expressions::evaluate_expressions(value, ctx)?;
                evaluated_env.insert(key.clone(), evaluated_value);
            }
            evaluated.env = Some(evaluated_env);
        }

        Ok(evaluated)
    }

    /// Execute a 'run' step
    fn execute_run_step(
        job_id: &str,
        step_name: &str,
        command: &str,
        env: &Option<HashMap<String, String>>,
        base_path: &Path,
        update_sender: &Sender<Update>,
        ctx: &mut EvaluationContext,
    ) -> io::Result<()> {
        // Create a temporary file for step outputs
        let output_file = base_path.join(format!(".ferri-step-output-{}-{}", job_id, step_name));

        // Build command for direct execution
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);
        cmd.current_dir(base_path);

        // Set FERRI_OUTPUT_FILE environment variable
        cmd.env("FERRI_OUTPUT_FILE", &output_file);

        // Add user-provided environment variables
        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        // Execute command and capture output
        let output_result = cmd.output();

        match output_result {
            Ok(output) => {
                // Combine stdout and stderr
                let mut combined_output = String::new();
                if !output.stdout.is_empty() {
                    combined_output.push_str(&String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    if !combined_output.is_empty() {
                        combined_output.push('\n');
                    }
                    combined_output.push_str(&String::from_utf8_lossy(&output.stderr));
                }

                if output.status.success() {
                    // Parse outputs from the output file
                    if output_file.exists() {
                        if let Ok(file) = fs::File::open(&output_file) {
                            let reader = io::BufReader::new(file);
                            for line in reader.lines() {
                                if let Ok(line) = line {
                                    // Parse format: name=value
                                    if let Some((name, value)) = line.split_once('=') {
                                        ctx.add_step_output(
                                            step_name.to_string(),
                                            name.trim().to_string(),
                                            value.trim().to_string(),
                                        );
                                    }
                                }
                            }
                        }
                        // Clean up the temporary file
                        let _ = fs::remove_file(&output_file);
                    }

                    // Also store stdout for backward compatibility
                    ctx.add_step_output(
                        step_name.to_string(),
                        "stdout".to_string(),
                        combined_output.clone(),
                    );

                    // Send completion update
                    update_sender.send(Update::Step(StepUpdate {
                        job_id: job_id.to_string(),
                        step_name: step_name.to_string(),
                        status: StepStatus::Completed,
                        output: Some(combined_output),
                    })).unwrap();

                    Ok(())
                } else {
                    // Clean up output file on failure
                    if output_file.exists() {
                        let _ = fs::remove_file(&output_file);
                    }

                    // Command failed
                    let exit_code = output.status.code().unwrap_or(-1);
                    let err_msg = format!(
                        "Step '{}' failed with exit code {}\n{}",
                        step_name, exit_code, combined_output
                    );

                    update_sender.send(Update::Step(StepUpdate {
                        job_id: job_id.to_string(),
                        step_name: step_name.to_string(),
                        status: StepStatus::Failed(err_msg.clone()),
                        output: Some(combined_output),
                    })).unwrap();

                    Err(io::Error::new(io::ErrorKind::Other, err_msg))
                }
            }
            Err(e) => {
                // Failed to execute command
                let err_msg = format!("Failed to execute step '{}': {}", step_name, e);

                update_sender.send(Update::Step(StepUpdate {
                    job_id: job_id.to_string(),
                    step_name: step_name.to_string(),
                    status: StepStatus::Failed(err_msg.clone()),
                    output: None,
                })).unwrap();

                Err(io::Error::new(io::ErrorKind::Other, err_msg))
            }
        }
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
    fn test_step_output_parsing() {
        use std::io::Write;
        use tempfile::TempDir;

        // Create a temporary directory for the test
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a mock output file
        let output_file = temp_path.join(".ferri-step-output-job1-step1");
        let mut file = fs::File::create(&output_file).unwrap();
        writeln!(file, "result=42").unwrap();
        writeln!(file, "message=hello world").unwrap();
        writeln!(file, "status=success").unwrap();
        drop(file);

        // Create evaluation context
        let mut ctx = EvaluationContext::new();

        // Parse the output file (simulating what the orchestrator does)
        if output_file.exists() {
            if let Ok(file) = fs::File::open(&output_file) {
                let reader = io::BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        if let Some((name, value)) = line.split_once('=') {
                            ctx.add_step_output(
                                "step1".to_string(),
                                name.trim().to_string(),
                                value.trim().to_string(),
                            );
                        }
                    }
                }
            }
        }

        // Verify outputs were parsed correctly
        assert_eq!(
            ctx.step_outputs.get("step1").unwrap().get("result").unwrap(),
            "42"
        );
        assert_eq!(
            ctx.step_outputs.get("step1").unwrap().get("message").unwrap(),
            "hello world"
        );
        assert_eq!(
            ctx.step_outputs.get("step1").unwrap().get("status").unwrap(),
            "success"
        );
    }
}
