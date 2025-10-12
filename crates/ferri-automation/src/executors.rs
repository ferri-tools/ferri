use anyhow::Result;
use crate::models::Workload;
use std::process::Command;

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
    fn execute(&self, workload: &Workload, context: ExecutionContext) -> Result<ExecutionHandle>;
    fn get_status(&self, handle: &ExecutionHandle) -> Result<ExecutionStatus>;
    fn get_logs(&self, handle: &ExecutionHandle) -> Result<Box<dyn std::io::Read>>;
    fn cleanup(&self, context: ExecutionContext) -> Result<()>;
}

pub struct Workspace;

// --- Process Executor ---

/// An executor that runs commands as local child processes.
pub struct ProcessExecutor;

impl Executor for ProcessExecutor {
    fn prepare(&self, _workload: &Workload, _workspace: &Workspace) -> Result<ExecutionContext> {
        // For ProcessExecutor, preparation is a no-op.
        Ok(ExecutionContext)
    }

    fn execute(&self, workload: &Workload, _context: ExecutionContext) -> Result<ExecutionHandle> {
        let mut command = Command::new(&workload.command);

        // ** SECURITY: Clear inherited environment variables **
        command.env_clear();

        // Inject environment variables from the workload
        if let Some(env_vars) = &workload.env {
            for (key, value) in env_vars {
                command.env(key, value);
            }
        }

        // Spawn the child process
        let child = command.spawn()?;

        // Return the process ID as the execution handle
        Ok(ExecutionHandle(child.id().to_string()))
    }

    fn get_status(&self, _handle: &ExecutionHandle) -> Result<ExecutionStatus> {
        // Placeholder implementation
        Ok(ExecutionStatus::Running)
    }

    fn get_logs(&self, _handle: &ExecutionHandle) -> Result<Box<dyn std::io::Read>> {
        // Placeholder implementation
        let empty: &[u8] = &[];
        Ok(Box::new(empty))
    }

    fn cleanup(&self, _context: ExecutionContext) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }
}