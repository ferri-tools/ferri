use std::collections::HashMap;
use std::io;
use std::path::Path;
use crate::flow::Job;
use std::process::Command;

pub trait Executor {
    fn execute(
        &self,
        job: &Job,
        base_path: &Path,
        secrets: &HashMap<String, String>,
    ) -> io::Result<ExecutionHandle>;
}

#[derive(Debug)]
pub struct ExecutionHandle(pub String);

pub struct ExecutorRegistry {
    executors: HashMap<String, Box<dyn Executor + Send + Sync>>,
}

impl ExecutorRegistry {
    pub fn new() -> Self {
        let mut executors: HashMap<String, Box<dyn Executor + Send + Sync>> = HashMap::new();
        executors.insert("process".to_string(), Box::new(ProcessExecutor));
        Self { executors }
    }

    pub fn get(&self, name: &str) -> Option<&(dyn Executor + Send + Sync)> {
        self.executors.get(name).map(|e| e.as_ref())
    }
}

struct ProcessExecutor;

impl Executor for ProcessExecutor {
    fn execute(
        &self,
        job: &Job,
        base_path: &Path,
        _secrets: &HashMap<String, String>,
    ) -> io::Result<ExecutionHandle> {
        for (step_idx, step) in job.steps.iter().enumerate() {
            if let Some(run_command) = &step.run {
                let mut cmd = Command::new("sh");
                cmd.arg("-c").arg(run_command);
                cmd.current_dir(base_path);

                let status = cmd.status()?; // This executes the command and waits for it

                if !status.success() {
                    let step_name = step.name.clone().unwrap_or_else(|| format!("step-{}", step_idx));
                    let err_msg = format!(
                        "Step '{}' failed with exit code {}",
                        step_name,
                        status.code().unwrap_or(1)
                    );
                    return Err(io::Error::new(io::ErrorKind::Other, err_msg));
                }
            }
        }
        // If all steps succeeded, return a success handle.
        Ok(ExecutionHandle("process-succeeded".to_string()))
    }
}