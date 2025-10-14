use std::collections::HashMap;
use std::io;
use std::path::Path;
use crate::flow::Job;

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

use std::process::Command;

struct ProcessExecutor;

impl Executor for ProcessExecutor {
    fn execute(
        &self,
        job: &Job,
        base_path: &Path,
        _secrets: &HashMap<String, String>,
    ) -> io::Result<ExecutionHandle> {
        let mut cmd = Command::new("echo");
        cmd.arg("Hello from the process executor!");
        cmd.current_dir(base_path);
        let child = cmd.spawn()?;
        Ok(ExecutionHandle(child.id().to_string()))
    }
}
