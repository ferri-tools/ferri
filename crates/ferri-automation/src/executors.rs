use anyhow::Result;
use crate::models::Workload;

/// A standardized representation of an execution's status.
pub enum ExecutionStatus {
    Running,
    Succeeded,
    Failed,
}

/// A handle to a running execution, allowing for status checks and log retrieval.
pub struct ExecutionHandle(String);

/// Contains handles to resources prepared for execution.
pub struct ExecutionContext;

/// The core trait defining the contract for all executors.
pub trait Executor {
    fn prepare(&self, workload: &Workload, workspace: &Workspace) -> Result<ExecutionContext>;
    fn execute(&self, context: ExecutionContext) -> Result<ExecutionHandle>;
    fn get_status(&self, handle: &ExecutionHandle) -> Result<ExecutionStatus>;
    fn get_logs(&self, handle: &ExecutionHandle) -> Result<Box<dyn std::io::Read>>;
    fn cleanup(&self, context: ExecutionContext) -> Result<()>;
}

pub struct Workspace;