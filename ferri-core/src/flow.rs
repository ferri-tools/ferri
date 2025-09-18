//! Core logic for parsing and executing AI pipelines from YAML files.

mod tui;

use crate::execute::{ExecutionArgs, PreparedCommand};
use crossbeam_channel::Sender;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
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
    
    // --- New comprehensive logger ---
    let log_path = base_path.join(".ferri").join("flow_run.log");
    let mut log_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)?;
    writeln!(log_file, "--- Starting flow: {} ---", pipeline.name)?;
    // ---

    for step in &pipeline.steps {
        let sender_clone = update_sender.clone();
        
        sender_clone.send(StepUpdate {
            name: step.name.clone(),
            status: StepStatus::Running,
            output: None,
        }).unwrap();

        writeln!(log_file, "\n--- Step '{}': Starting ---", step.name)?;

        let mut input_data: Option<Vec<u8>> = None;
        if let Some(input_source) = &step.input {
            writeln!(log_file, "Reading input from '{}'.", input_source)?;
            let mut combined_input = Vec::new();
            let sources: Vec<&str> = input_source.split(',').map(|s| s.trim()).collect();
            for source in sources {
                let outputs = step_outputs.lock().unwrap();
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

        let (prepared_command, secrets) = match &step.kind {
            StepKind::Model(model_step) => {
                writeln!(log_file, "Preparing model command for provider '{}'.", model_step.model)?;
                let final_prompt = if let Some(input) = &input_data {
                    format!("{}\n\n{}", String::from_utf8_lossy(input), model_step.prompt)
                } else {
                    model_step.prompt.clone()
                };
                let exec_args = ExecutionArgs {
                    model: Some(model_step.model.clone()),
                    use_context: false,
                    output_file: None,
                    command_with_args: vec![final_prompt],
                };
                crate::execute::prepare_command(base_path, &exec_args)?
            }
            StepKind::Process(process_step) => {
                writeln!(log_file, "Preparing process command '{}'.", process_step.process)?;
                let mut cmd = Command::new("sh");
                cmd.arg("-c").arg(&process_step.process);
                (PreparedCommand::Local(cmd), HashMap::new())
            }
        };

        let final_output = match prepared_command {
            PreparedCommand::Local(mut command) => {
                writeln!(log_file, "Executing local command.")?;
                let mut child = command
                    .envs(secrets)
                    .stdin(Stdio::piped()) // Pipe stdin
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?;

                // Write input data to the child's stdin in a separate thread
                if let Some(input_data) = input_data {
                    let mut stdin = child.stdin.take().unwrap();
                    thread::spawn(move || {
                        stdin.write_all(&input_data).unwrap();
                    });
                }

                let stdout = BufReader::new(child.stdout.take().unwrap());
                let stderr = BufReader::new(child.stderr.take().unwrap());
                let mut accumulated_stdout = Vec::new();

                let sender_clone_err = sender_clone.clone();
                let step_name_clone_err = step.name.clone();
                let mut log_file_err = log_file.try_clone()?;
                let stderr_thread = thread::spawn(move || {
                    for line in stderr.lines() {
                        let line = line.unwrap_or_default();
                        writeln!(log_file_err, "STDERR: {}", line).ok();
                        sender_clone_err.send(StepUpdate { name: step_name_clone_err.clone(), status: StepStatus::Running, output: Some(line) }).unwrap();
                    }
                });

                for line in stdout.lines() {
                    let line = line?;
                    writeln!(log_file, "STDOUT: {}", line)?;
                    sender_clone.send(StepUpdate { name: step.name.clone(), status: StepStatus::Running, output: Some(line.clone()) }).unwrap();
                    accumulated_stdout.extend_from_slice(line.as_bytes());
                    accumulated_stdout.push(b'\n');
                }
                
                stderr_thread.join().unwrap();
                let status = child.wait()?;
                writeln!(log_file, "Local command finished with status: {}", status)?;
                std::process::Output {
                    status,
                    stdout: accumulated_stdout,
                    stderr: vec![],
                }
            }
            PreparedCommand::Remote(request) => {
                writeln!(log_file, "Sending remote API request.")?;
                let response = request.send().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                let status = response.status();
                writeln!(log_file, "Received API response with status: {}", status)?;

                if !status.is_success() {
                    let body = response.bytes().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    let stderr = if let Ok(json_err) = serde_json::from_slice::<serde_json::Value>(&body) {
                        if let Some(msg) = json_err["error"]["message"].as_str() {
                            format!("API Error: {}", msg)
                        } else { String::from_utf8_lossy(&body).to_string() }
                    } else { String::from_utf8_lossy(&body).to_string() };
                    writeln!(log_file, "API error: {}", stderr)?;
                    sender_clone.send(StepUpdate { name: step.name.clone(), status: StepStatus::Failed(stderr.clone()), output: None }).unwrap();
                    return Err(io::Error::new(io::ErrorKind::Other, stderr));
                }

                let mut reader = BufReader::new(response);
                let mut accumulated_stdout = Vec::new();
                let mut buffer = Vec::new();

                loop {
                    let mut chunk = [0; 1024];
                    let bytes_read = reader.read(&mut chunk)?;
                    writeln!(log_file, "Read {} bytes from stream.", bytes_read)?;
                    if bytes_read == 0 {
                        writeln!(log_file, "Stream ended.")?;
                        break;
                    }
                    buffer.extend_from_slice(&chunk[..bytes_read]);

                    // This logic finds complete JSON objects in the buffer
                    let mut de = serde_json::Deserializer::from_slice(&buffer).into_iter::<serde_json::Value>();
                    while let Some(Ok(json_value)) = de.next() {
                        writeln!(log_file, "Parsed JSON value: {:?}", json_value)?;
                        if let serde_json::Value::Array(chunks) = json_value {
                            for chunk in chunks {
                                if let Some(text) = chunk["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                                    writeln!(log_file, "Extracted text: {}", text)?;
                                    sender_clone.send(StepUpdate { name: step.name.clone(), status: StepStatus::Running, output: Some(text.to_string()) }).unwrap();
                                    accumulated_stdout.extend_from_slice(text.as_bytes());
                                }
                            }
                        }
                    }
                    // Keep the remainder of the buffer that wasn't parsed
                    buffer = buffer[de.byte_offset()..].to_vec();
                }

                writeln!(log_file, "Finished processing remote stream.")?;
                std::process::Output {
                    status: Command::new("true").status()?,
                    stdout: accumulated_stdout,
                    stderr: vec![],
                }
            }
        };

        if !final_output.status.success() {
            let err_msg = String::from_utf8_lossy(&final_output.stderr).to_string();
            writeln!(log_file, "Step failed with message: {}", err_msg)?;
            sender_clone.send(StepUpdate { name: step.name.clone(), status: StepStatus::Failed(err_msg), output: None }).unwrap();
            return Err(io::Error::new(io::ErrorKind::Other, format!("Step '{}' failed.", step.name)));
        }

        if let Some(output_path) = &step.output {
            writeln!(log_file, "Writing output to '{}'.", output_path)?;
            fs::write(output_path, &final_output.stdout)?;
        }
        
        let mut outputs = step_outputs.lock().unwrap();
        outputs.insert(step.name.clone(), final_output.stdout);

        writeln!(log_file, "--- Step '{}': Completed Successfully ---", step.name)?;
        sender_clone.send(StepUpdate { name: step.name.clone(), status: StepStatus::Completed, output: None }).unwrap();
    }

    Ok(())
}


pub fn show_pipeline(pipeline: &Pipeline) -> io::Result<()> {
    tui::run_tui(pipeline)
}
