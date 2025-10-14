//! Core logic for parsing and executing AI workflows from ferri-flow.yml files.
//!
//! This module implements the ferri-flow.yml schema specification, which supports:
//! - Declarative workflow definitions with metadata and versioning
//! - Parallel job execution with dependency management
//! - Expression syntax for context passing
//! - Workspaces for explicit filesystem I/O
//! - Retry strategies with exponential backoff

use crate::execute::{ExecutionArgs, SharedArgs};
use crate::jobs;
use ferri_core::context;
use clap::Parser;
use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{fs, io, thread};

// --- Data structures for real-time updates ---
#[derive(Clone, Debug)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct StepUpdate {
    pub job_id: String,
    pub step_name: String,
    pub status: StepStatus,
    pub output: Option<String>,
}

#[derive(Clone, Debug)]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct JobUpdate {
    pub job_id: String,
    pub status: JobStatus,
}

#[derive(Clone, Debug)]
pub enum Update {
    Job(JobUpdate),
    Step(StepUpdate),
}
// ---

// --- Top-Level Schema ---

/// The root document structure for ferri-flow.yml
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlowDocument {
    /// Schema version (e.g., "ferri.flow/v1alpha1")
    pub api_version: String,

    /// Type of document (must be "Flow")
    pub kind: String,

    /// Identifying metadata
    pub metadata: Metadata,

    /// The workflow specification
    pub spec: FlowSpec,
}

/// Metadata for identifying and organizing workflows
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Metadata {
    /// Unique kebab-case name
    pub name: String,

    /// Optional key-value labels for organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,

    /// Optional key-value annotations for metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

/// The workflow specification
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FlowSpec {
    /// Input parameters for the flow
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputs: Option<HashMap<String, Input>>,

    /// Shared storage volumes for filesystem I/O
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<Vec<Workspace>>,

    /// Map of job definitions (job-id -> Job)
    pub jobs: HashMap<String, Job>,
}

// --- Input Definition ---

/// Input parameter definition
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    /// Human-readable description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Data type: "string", "number", or "boolean"
    #[serde(rename = "type")]
    pub input_type: InputType,

    /// Default value if not provided
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_yaml::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    String,
    Number,
    Boolean,
}

// --- Workspace Definition ---

/// Workspace for shared filesystem storage
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Workspace {
    /// Unique workspace name within the flow
    pub name: String,
}

// --- Job Definition ---

/// A job represents a collection of steps that run on a single runner
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Job {
    /// Human-readable display name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Runner environment (e.g., "ubuntu-latest")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runs_on: Option<String>,

    /// Job IDs that must complete before this job starts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub needs: Option<Vec<String>>,

    /// Sequential steps to execute
    pub steps: Vec<Step>,
}

// --- Step Definition ---

/// A single step within a job
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Step {
    /// Unique step identifier (kebab-case)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Human-readable step name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Shell command to execute (mutually exclusive with 'uses')
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,

    /// Reusable action to execute (mutually exclusive with 'run')
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<String>,
    
    /// Declared output artifacts or files the task is expected to produce.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Vec<String>>,

    /// Input parameters for the action specified by 'uses'
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub with: Option<HashMap<String, serde_yaml::Value>>,

    /// Environment variables for this step
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,

    /// Workspaces to mount into this step
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<Vec<StepWorkspace>>,

    /// Retry policy for this step
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "retryStrategy")]
    pub retry_strategy: Option<RetryStrategy>,
}

/// Workspace mount configuration for a step
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepWorkspace {
    /// Name of the workspace to mount (must exist in spec.workspaces)
    pub name: String,

    /// Absolute path where workspace will be mounted
    pub mount_path: String,

    /// Mount in read-only mode
    #[serde(default)]
    pub read_only: bool,
}

// --- Retry Strategy ---

/// Retry configuration for a step
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RetryStrategy {
    /// Maximum number of retries (default: 0)
    #[serde(default)]
    pub limit: u32,

    /// Condition for retrying
    #[serde(default, rename = "retryPolicy")]
    pub retry_policy: RetryPolicy,

    /// Exponential backoff configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backoff: Option<Backoff>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum RetryPolicy {
    OnFailure,  // Retry on non-zero exit code
    OnError,    // Retry on infrastructure errors
    Always,     // Retry on any failure
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy::OnFailure
    }
}

/// Exponential backoff configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Backoff {
    /// Initial delay duration (e.g., "10s", "1m")
    pub duration: String,

    /// Multiplier for each retry (default: 2)
    #[serde(default = "default_backoff_factor")]
    pub factor: u32,

    /// Maximum delay duration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_duration: Option<String>,
}

fn default_backoff_factor() -> u32 {
    2
}

// --- Parser ---

/// Parse a ferri-flow.yml file
pub fn parse_flow_file(file_path: &Path) -> io::Result<FlowDocument> {
    let content = fs::read_to_string(file_path)?;
    let flow: FlowDocument = serde_yaml::from_str(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    // Validate the document
    validate_flow(&flow)?;

    Ok(flow)
}

/// Validate a flow document
fn validate_flow(flow: &FlowDocument) -> io::Result<()> {
    // Check kind
    if flow.kind != "Flow" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid kind '{}', expected 'Flow'", flow.kind)
        ));
    }

    // Check apiVersion format (basic check)
    if !flow.api_version.contains("ferri.flow/") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid apiVersion '{}', expected format 'ferri.flow/vX'", flow.api_version)
        ));
    }

    // Validate jobs is not empty
    if flow.spec.jobs.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Flow must define at least one job"
        ));
    }

    // Validate each job
    for (job_id, job) in &flow.spec.jobs {
        validate_job(job_id, job)?;
    }

    // Validate job dependencies
    validate_job_dependencies(&flow.spec.jobs)?;

    // Validate workspace references
    if let Some(workspaces) = &flow.spec.workspaces {
        validate_workspace_references(&flow.spec.jobs, workspaces)?;
    }

    Ok(())
}

/// Validate a single job
fn validate_job(job_id: &str, job: &Job) -> io::Result<()> {
    // Check job has at least one step
    if job.steps.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Job '{}' must have at least one step", job_id)
        ));
    }

    // Validate each step
    for (idx, step) in job.steps.iter().enumerate() {
        // Must have either 'run' or 'uses', but not both
        match (&step.run, &step.uses) {
            (None, None) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Step {} in job '{}' must have either 'run' or 'uses'", idx, job_id)
                ));
            }
            (Some(_), Some(_)) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Step {} in job '{}' cannot have both 'run' and 'uses'", idx, job_id)
                ));
            }
            _ => {}
        }
    }

    Ok(())
}

/// Validate job dependencies don't have cycles
fn validate_job_dependencies(jobs: &HashMap<String, Job>) -> io::Result<()> {
    for (job_id, job) in jobs {
        if let Some(needs) = &job.needs {
            for dep in needs {
                // Check dependency exists
                if !jobs.contains_key(dep) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Job '{}' depends on non-existent job '{}'", job_id, dep)
                    ));
                }

                // Check for self-dependency
                if dep == job_id {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Job '{}' cannot depend on itself", job_id)
                    ));
                }
            }
        }
    }

    // TODO: Add cycle detection

    Ok(())
}

/// Validate workspace references
fn validate_workspace_references(
    jobs: &HashMap<String, Job>,
    workspaces: &[Workspace],
) -> io::Result<()> {
    let workspace_names: Vec<&str> = workspaces.iter().map(|w| w.name.as_str()).collect();

    for (job_id, job) in jobs {
        for step in &job.steps {
            if let Some(step_workspaces) = &step.workspaces {
                for sw in step_workspaces {
                    if !workspace_names.contains(&sw.name.as_str()) {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "Step in job '{}' references non-existent workspace '{}'",
                                job_id, sw.name
                            )
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

// --- Legacy Support (kept for backward compatibility) ---

/// Legacy pipeline structure (deprecated)
#[derive(Debug, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub steps: Vec<LegacyStep>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LegacyStep {
    pub name: String,
    pub command: String,
    pub input: Option<String>,
    pub output: Option<String>,
}

pub fn parse_pipeline_file(file_path: &Path) -> io::Result<Pipeline> {
    let content = fs::read_to_string(file_path)?;
    serde_yaml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

// Helper struct to parse command arguments using clap
#[derive(Parser, Debug)]
struct StepCommandArgs {
    #[clap(flatten)]
    shared: SharedArgs,
}

pub fn run_pipeline(
    base_path: &Path,
    pipeline: &Pipeline,
    update_sender: Sender<StepUpdate>,
) -> io::Result<()> {
    // This map now tracks the explicit output file path for each step.
    let step_output_files = Arc::new(Mutex::new(HashMap::<String, PathBuf>::new()));

    for step in &pipeline.steps {
        let sender_clone = update_sender.clone();
        sender_clone
            .send(StepUpdate {
                job_id: "legacy".to_string(),
                step_name: step.name.clone(),
                status: StepStatus::Running,
                output: None,
            })
            .unwrap();

        // --- Context Management (The Correct Architecture) ---
        context::clear_context(base_path)?;
        let mut input_paths = Vec::new();
        if let Some(input_sources) = &step.input {
            let sources: Vec<&str> = input_sources.split(',').map(|s| s.trim()).collect();
            for source in sources {
                // Check if the input is the output of a previous step.
                if let Some(path) = step_output_files.lock().unwrap().get(source) {
                    input_paths.push(path.clone());
                } else if Path::new(source).exists() { // Otherwise, check the filesystem.
                    input_paths.push(PathBuf::from(source));
                } else {
                    return Err(io::Error::new(io::ErrorKind::NotFound, format!("Input '{}' not found for step '{}'", source, step.name)));
                }
            }
        }

        if !input_paths.is_empty() {
            context::add_to_context(base_path, input_paths)?;
        }

        // --- Command Preparation ---
        let command_parts = shell_words::split(&step.command)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

        let parsed_args = StepCommandArgs::try_parse_from(command_parts)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

        let exec_args = ExecutionArgs {
            model: parsed_args.shared.model,
            use_context: parsed_args.shared.ctx,
            output_file: parsed_args.shared.output,
            command_with_args: parsed_args.shared.command,
            streaming: true,
        };

        let (prepared_command, secrets) = crate::execute::prepare_command(base_path, &exec_args)?;

        // Clone the output file path before it's moved into the job.
        let output_file_for_registration = exec_args.output_file.clone();

        // --- Job Submission ---
        let job = jobs::submit_job(
            base_path,
            prepared_command,
            secrets,
            &exec_args.command_with_args,
            None, // Stdin is not used for inter-step data transfer
            exec_args.output_file,
        )?;

        let job_id = job.id;

        // --- Synchronous Polling ---
        loop {
            thread::sleep(std::time::Duration::from_millis(500));
            let jobs = jobs::list_jobs(base_path)?;
            if let Some(job_instance) = jobs.iter().find(|j| j.id == job_id) {
                match job_instance.status.as_str() {
                    "Completed" => {
                        // If this step produced an output file, record it for subsequent steps.
                        if let Some(output_path) = output_file_for_registration {
                            step_output_files.lock().unwrap().insert(step.name.clone(), output_path);
                        }
                        sender_clone.send(StepUpdate {
                            job_id: "legacy".to_string(),
                            step_name: step.name.clone(),
                            status: StepStatus::Completed,
                            output: None
                        }).unwrap();
                        break;
                    }
                    "Failed" => {
                        let err_msg = format!("Step '{}' failed. See job '{}' for details.", step.name, job_id);
                        sender_clone.send(StepUpdate {
                            job_id: "legacy".to_string(),
                            step_name: step.name.clone(),
                            status: StepStatus::Failed(err_msg.clone()),
                            output: None
                        }).unwrap();
                        return Err(io::Error::new(io::ErrorKind::Other, err_msg));
                    }
                    _ => { // Still running
                        let output = jobs::get_job_output(base_path, &job_id)?;
                        sender_clone.send(StepUpdate {
                            job_id: "legacy".to_string(),
                            step_name: step.name.clone(),
                            status: StepStatus::Running,
                            output: Some(output)
                        }).unwrap();
                    }
                }
            } else {
                // Job not found, maybe it finished and was archived?
                // For this loop, we assume it's completed.
                sender_clone.send(StepUpdate {
                    job_id: "legacy".to_string(),
                    step_name: step.name.clone(),
                    status: StepStatus::Completed,
                    output: None
                }).unwrap();
                break;
            }
        }
    }
    Ok(())
}

pub fn show_pipeline(_pipeline: &Pipeline) -> io::Result<()> {
    // The TUI logic is now internal to the `ferri-cli` crate.
    // This function can be a placeholder or removed if `flow show` is handled entirely in `main.rs`.
    println!("'flow show' is not implemented in this context.");
    Ok(())
}
