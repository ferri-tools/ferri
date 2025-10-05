//! Job orchestrator for ferri-flow execution
//!
//! This module handles:
//! - Parallel job execution
//! - Dependency resolution via topological sort
//! - Context management and expression evaluation
//! - Real-time status updates

use crate::execute::PreparedCommand;
use crate::expressions::{self, EvaluationContext};
use crate::flow::{FlowDocument, Job, Step, StepUpdate, StepStatus};
use crate::jobs;
use crossbeam_channel::Sender;
use std::collections::{HashMap, VecDeque};
use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Orchestrator state for flow execution
pub struct FlowOrchestrator {
    flow: FlowDocument,
    base_path: std::path::PathBuf,
    update_sender: Sender<StepUpdate>,
    runtime_inputs: HashMap<String, String>,
}

impl FlowOrchestrator {
    pub fn new(
        flow: FlowDocument,
        base_path: &Path,
        update_sender: Sender<StepUpdate>,
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
        for handle in handles {
            if let Err(e) = handle.join() {
                errors.push(format!("Job thread panicked: {:?}", e));
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
        update_sender: Sender<StepUpdate>,
        runtime_inputs: &HashMap<String, String>,
        job_outputs: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
        _flow: &FlowDocument,
    ) -> io::Result<()> {
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
        update_sender: &Sender<StepUpdate>,
        ctx: &mut EvaluationContext,
    ) -> io::Result<()> {
        // Send running status
        update_sender.send(StepUpdate {
            job_id: job_id.to_string(),
            step_name: step_name.to_string(),
            status: StepStatus::Running,
            output: None,
        }).unwrap();

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
        update_sender: &Sender<StepUpdate>,
        ctx: &mut EvaluationContext,
    ) -> io::Result<()> {
        // For now, execute as a simple shell command
        // TODO: Integrate with proper job system, handle env vars, workspaces, etc.

        // Build command
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);

        // Add environment variables
        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        let prepared_cmd = PreparedCommand::Local(cmd, None);
        let original_command = vec![command.to_string()];

        let job = jobs::submit_job(
            base_path,
            prepared_cmd,
            HashMap::new(),
            &original_command,
            None,
            None,
        )?;

        let background_job_id = job.id;

        // Poll until completion
        loop {
            thread::sleep(Duration::from_millis(500));
            let jobs_list = jobs::list_jobs(base_path)?;

            if let Some(bg_job) = jobs_list.iter().find(|j| j.id == background_job_id) {
                match bg_job.status.as_str() {
                    "Completed" => {
                        // Get output
                        let output = jobs::get_job_output(base_path, &background_job_id)?;

                        // TODO: Parse ferri-runtime set-output commands from output
                        // For now, just store the full output
                        if let Some(step_id) = ctx.step_outputs.keys().next() {
                            ctx.add_step_output(
                                step_id.to_string(),
                                "stdout".to_string(),
                                output.clone(),
                            );
                        }

                        update_sender.send(StepUpdate {
                            job_id: job_id.to_string(),
                            step_name: step_name.to_string(),
                            status: StepStatus::Completed,
                            output: Some(output),
                        }).unwrap();

                        break;
                    }
                    "Failed" => {
                        let err_msg = format!("Step '{}' failed", step_name);
                        update_sender.send(StepUpdate {
                            job_id: job_id.to_string(),
                            step_name: step_name.to_string(),
                            status: StepStatus::Failed(err_msg.clone()),
                            output: None,
                        }).unwrap();

                        return Err(io::Error::new(io::ErrorKind::Other, err_msg));
                    }
                    _ => {
                        // Still running
                        let output = jobs::get_job_output(base_path, &background_job_id)?;
                        update_sender.send(StepUpdate {
                            job_id: job_id.to_string(),
                            step_name: step_name.to_string(),
                            status: StepStatus::Running,
                            output: Some(output),
                        }).unwrap();
                    }
                }
            } else {
                // Job not found - assume completed
                update_sender.send(StepUpdate {
                    job_id: job_id.to_string(),
                    step_name: step_name.to_string(),
                    status: StepStatus::Completed,
                    output: None,
                }).unwrap();
                break;
            }
        }

        Ok(())
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
}
