//! Core logic for parsing and executing AI pipelines from YAML files.

mod tui;

use crate::execute::{ExecutionArgs, SharedArgs};
use crate::jobs;
use crate::context;
use clap::Parser;
use crossbeam_channel::Sender;
use serde::Deserialize;
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
    pub name: String,
    pub status: StepStatus,
    pub output: Option<String>,
}
// ---

#[derive(Debug, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Step {
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
                name: step.name.clone(),
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
            if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
                match job.status.as_str() {
                    "Completed" => {
                        // If this step produced an output file, record it for subsequent steps.
                        if let Some(output_path) = output_file_for_registration {
                            step_output_files.lock().unwrap().insert(step.name.clone(), output_path);
                        }
                        sender_clone.send(StepUpdate { name: step.name.clone(), status: StepStatus::Completed, output: None }).unwrap();
                        break;
                    }
                    "Failed" => {
                        let err_msg = format!("Step '{}' failed. See job '{}' for details.", step.name, job_id);
                        sender_clone.send(StepUpdate { name: step.name.clone(), status: StepStatus::Failed(err_msg.clone()), output: None }).unwrap();
                        return Err(io::Error::new(io::ErrorKind::Other, err_msg));
                    }
                    _ => { // Still running
                        let output = jobs::get_job_output(base_path, &job_id)?;
                        sender_clone.send(StepUpdate { name: step.name.clone(), status: StepStatus::Running, output: Some(output) }).unwrap();
                    }
                }
            }
        }
    }
    Ok(())
}


pub fn show_pipeline(pipeline: &Pipeline) -> io::Result<()> {
    tui::run_tui(pipeline)
}