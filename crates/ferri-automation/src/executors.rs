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
        let mut executors: HashMap<String, Box<dyn Executor + Send + Sync>> =HashMap::new();
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
        _job: &Job,
        _base_path: &Path,
        _secrets: &HashMap<String, String>,
    ) -> io::Result<ExecutionHandle> {
        // TODO: Implement process execution
        Ok(ExecutionHandle("dummy-handle".to_string()))
    }
}