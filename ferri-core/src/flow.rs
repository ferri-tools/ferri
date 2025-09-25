//! Core logic for parsing and executing AI pipelines from YAML files.

mod tui;

use crossbeam_channel::Sender;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path};
use std::process::{Command, Stdio};
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

use crate::jobs;

pub fn run_pipeline(
    base_path: &Path,
    pipeline: &Pipeline,
    update_sender: Sender<StepUpdate>,
) -> io::Result<()> {
    let step_outputs = Arc::new(Mutex::new(HashMap::<String, String>::new()));

    for step in &pipeline.steps {
        let sender_clone = update_sender.clone();
        sender_clone
            .send(StepUpdate {
                name: step.name.clone(),
                status: StepStatus::Running,
                output: None,
            })
            .unwrap();

        // --- Input Handling (T75) ---
        let mut input_data: Option<Vec<u8>> = None;
        if let Some(input_source_name) = &step.input {
            let job_id = step_outputs.lock().unwrap().get(input_source_name).cloned();
            if let Some(id) = job_id {
                // Use ferri yank equivalent to get output
                input_data = Some(
                    jobs::get_job_output(base_path, &id)?
                        .as_bytes()
                        .to_vec(),
                );
            } else if Path::new(input_source_name).exists() {
                input_data = Some(fs::read(input_source_name)?);
            }
        }

        // --- Command Construction (T73) ---
        let command_to_run = format!("ferri run -- {}", step.command);
        let mut command = Command::new("sh");
        command.arg("-c").arg(&command_to_run);

        // --- Job Submission ---
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(data) = input_data {
            let mut stdin = child.stdin.take().unwrap();
            thread::spawn(move || {
                stdin.write_all(&data).unwrap();
            });
        }

        let stdout = BufReader::new(child.stdout.take().unwrap());
        let job_id_line = stdout.lines().next().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Failed to get job ID from ferri run")
        })??;

        let job_id: String = job_id_line
            .split_whitespace()
            .last()
            .unwrap()
            .to_string();

        step_outputs
            .lock()
            .unwrap()
            .insert(step.name.clone(), job_id.clone());

        // --- Synchronous Polling (T74) ---
        loop {
            thread::sleep(std::time::Duration::from_millis(200));
            let jobs = jobs::list_jobs(base_path)?;
            if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
                match job.status.as_str() {
                    "Completed" => {
                        sender_clone
                            .send(StepUpdate {
                                name: step.name.clone(),
                                status: StepStatus::Completed,
                                output: None,
                            })
                            .unwrap();
                        break;
                    }
                    "Failed" => {
                        let err_msg = format!("Step '{}' failed.", step.name);
                        sender_clone
                            .send(StepUpdate {
                                name: step.name.clone(),
                                status: StepStatus::Failed(err_msg.clone()),
                                output: None,
                            })
                            .unwrap();
                        return Err(io::Error::new(io::ErrorKind::Other, err_msg));
                    }
                    _ => {
                        // Still running, maybe send an update
                        let output = jobs::get_job_output(base_path, &job_id)?;
                        sender_clone
                            .send(StepUpdate {
                                name: step.name.clone(),
                                status: StepStatus::Running,
                                output: Some(output),
                            })
                            .unwrap();
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



