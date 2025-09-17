//! Core logic for parsing and executing AI pipelines from YAML files.

mod tui;

use crate::execute::{ExecutionArgs, PreparedCommand};
use crossbeam_channel::Sender;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
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
    #[serde(flatten)]
    pub kind: StepKind,
    pub input: Option<String>,
    pub output: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum StepKind {
    Model(ModelStep),
    Process(ProcessStep),
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelStep {
    pub model: String,
    pub prompt: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProcessStep {
    pub process: String,
}

pub fn parse_pipeline_file(file_path: &Path) -> io::Result<Pipeline> {
    let content = fs::read_to_string(file_path)?;
    serde_yaml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn run_pipeline(
    base_path: &Path,
    pipeline: &Pipeline,
    update_sender: Sender<StepUpdate>,
) -> io::Result<()> {
    let step_outputs = Arc::new(Mutex::new(HashMap::<String, Vec<u8>>::new()));

    for step in &pipeline.steps {
        let step_clone = step.clone();
        let base_path_clone = base_path.to_path_buf();
        let sender_clone = update_sender.clone();
        let outputs_clone = Arc::clone(&step_outputs);

        sender_clone.send(StepUpdate {
            name: step_clone.name.clone(),
            status: StepStatus::Running,
            output: None,
        }).unwrap();

        let handle = thread::spawn(move || -> io::Result<()> {
            let mut input_data: Option<Vec<u8>> = None;
            if let Some(input_source) = &step_clone.input {
                let mut combined_input = Vec::new();
                let sources: Vec<&str> = input_source.split(',').map(|s| s.trim()).collect();
                for source in sources {
                    let outputs = outputs_clone.lock().unwrap();
                    if let Some(output) = outputs.get(source) {
                        combined_input.extend_from_slice(output);
                        combined_input.push(b'\n');
                    } else if Path::new(source).exists() {
                        combined_input.extend_from_slice(&fs::read(source)?);
                        combined_input.push(b'\n');
                    }
                }
                if !combined_input.is_empty() {
                    input_data = Some(combined_input);
                }
            }

            let (prepared_command, secrets) = match &step_clone.kind {
                StepKind::Model(model_step) => {
                    let final_prompt = if let Some(input) = &input_data {
                        format!("{}\n\n{}", String::from_utf8_lossy(input), model_step.prompt)
                    } else {
                        model_step.prompt.clone()
                    };
                    let exec_args = ExecutionArgs {
                        model: Some(model_step.model.clone()),
                        use_context: false,
                        command_with_args: vec![final_prompt],
                    };
                    crate::execute::prepare_command(&base_path_clone, &exec_args)?
                }
                StepKind::Process(process_step) => {
                    let mut cmd = Command::new("sh");
                    cmd.arg("-c").arg(&process_step.process);
                    (PreparedCommand::Local(cmd), HashMap::new())
                }
            };

            let final_output = match prepared_command {
                PreparedCommand::Local(mut command) => {
                    let mut child = command.envs(secrets).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
                    let stdout = BufReader::new(child.stdout.take().unwrap());
                    let stderr = BufReader::new(child.stderr.take().unwrap());

                    let sender_clone_err = sender_clone.clone();
                    let step_name_clone_err = step_clone.name.clone();
                    let stderr_thread = thread::spawn(move || {
                        for line in stderr.lines() {
                            sender_clone_err.send(StepUpdate { name: step_name_clone_err.clone(), status: StepStatus::Running, output: Some(line.unwrap_or_default()) }).unwrap();
                        }
                    });

                    for line in stdout.lines() {
                        sender_clone.send(StepUpdate { name: step_clone.name.clone(), status: StepStatus::Running, output: Some(line?) }).unwrap();
                    }
                    
                    stderr_thread.join().unwrap();
                    child.wait_with_output()? 
                }
                PreparedCommand::Remote(request) => {
                    // Simplified for now, full streaming in next ticket
                    let response = request.send().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    let status = response.status();
                    let body = response.bytes().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    let exit_status = if status.is_success() { Command::new("true").status()? } else { Command::new("false").status()? };
                    std::process::Output { status: exit_status, stdout: body.to_vec(), stderr: vec![] }
                }
            };

            if !final_output.status.success() {
                let err_msg = String::from_utf8_lossy(&final_output.stderr).to_string();
                sender_clone.send(StepUpdate { name: step_clone.name.clone(), status: StepStatus::Failed(err_msg), output: None }).unwrap();
                return Err(io::Error::new(io::ErrorKind::Other, format!("Step '{}' failed.", step_clone.name)));
            }

            if let Some(output_path) = &step_clone.output {
                fs::write(output_path, &final_output.stdout)?;
            }
            
            let mut outputs = outputs_clone.lock().unwrap();
            outputs.insert(step_clone.name.clone(), final_output.stdout);

            sender_clone.send(StepUpdate { name: step_clone.name.clone(), status: StepStatus::Completed, output: None }).unwrap();
            Ok(())
        });

        handle.join().unwrap()?;
    }

    Ok(())
}

pub fn show_pipeline(pipeline: &Pipeline) -> io::Result<()> {
    tui::run_tui(pipeline)
}