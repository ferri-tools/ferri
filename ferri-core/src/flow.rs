//! Core logic for parsing and executing AI pipelines from YAML files.

use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
pub struct Step {
    pub name: String,
    #[serde(flatten)]
    pub kind: StepKind,
    pub input: Option<String>,
    pub output: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepKind {
    Model(ModelStep),
    Process(ProcessStep),
}

#[derive(Debug, Deserialize)]
pub struct ModelStep {
    pub model: String,
    pub prompt: String,
}

#[derive(Debug, Deserialize)]
pub struct ProcessStep {
    pub process: String,
}

pub fn parse_pipeline_file(file_path: &Path) -> io::Result<Pipeline> {
    let content = fs::read_to_string(file_path)?;
    serde_yaml::from_str(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_pipeline() {
        let yaml = r#"
name: "Test Pipeline"
steps:
  - name: "step-one"
    model:
      model: "gemma3"
      prompt: "Summarize this"
  - name: "step-two"
    process:
      process: "grep -i error"
"#;
        let result: Result<Pipeline, _> = serde_yaml::from_str(yaml);
        assert!(result.is_ok());
        let pipeline = result.unwrap();
        assert_eq!(pipeline.name, "Test Pipeline");
        assert_eq!(pipeline.steps.len(), 2);
        assert_eq!(pipeline.steps[0].name, "step-one");
        if let StepKind::Model(model_step) = &pipeline.steps[0].kind {
            assert_eq!(model_step.model, "gemma3");
        } else {
            panic!("Expected a model step");
        }
        if let StepKind::Process(process_step) = &pipeline.steps[1].kind {
            assert_eq!(process_step.process, "grep -i error");
        } else {
            panic!("Expected a process step");
        }
    }
}

    #[test]
    fn test_prepare_model_step_with_secret() {
        use crate::{initialize_project, models, secrets};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let base_path = dir.path();
        initialize_project(base_path).unwrap();
        secrets::set_secret(base_path, "MY_KEY", "test-secret").unwrap();
        let model = models::Model {
            alias: "gpt-test".to_string(),
            provider: "openai".to_string(),
            model_name: "gpt-4".to_string(),
            api_key_secret: Some("MY_KEY".to_string()),
            discovered: false,
        };
        models::add_model(base_path, model).unwrap();

        let model_step = ModelStep {
            model: "gpt-test".to_string(),
            prompt: "This is a test".to_string(),
        };

        let exec_args = ExecutionArgs {
            model: Some(model_step.model.clone()),
            use_context: false,
            command_with_args: vec![model_step.prompt.clone()],
        };

        let result = crate::execute::prepare_command(base_path, &exec_args);
        assert!(result.is_ok());
        let (_, secrets) = result.unwrap();
        assert!(secrets.contains_key("MY_KEY"));
        assert_eq!(secrets.get("MY_KEY").unwrap(), "test-secret");
    }

    #[test]
    fn test_show_pipeline_handles_multiline_prompt() {
        let pipeline = Pipeline {
            name: "Multiline Test".to_string(),
            steps: vec![
                Step {
                    name: "step-1".to_string(),
                    kind: StepKind::Model(ModelStep {
                        model: "gemma".to_string(),
                        prompt: "This is a very long prompt\nthat contains newlines\nand should be truncated.".to_string(),
                    }),
                    input: None,
                    output: None,
                }
            ],
        };
        // We can't test the `treetrunk` output directly, but we can test the input string
        // that `show_pipeline` generates.
        let mut viz_input = format!("{}\n", pipeline.name);
        for step in &pipeline.steps {
            let mut step_details = match &step.kind {
                StepKind::Model(model_step) => {
                    let mut prompt = model_step.prompt.replace('\n', " ");
                    if prompt.len() > 40 {
                        prompt.truncate(37);
                        prompt.push_str("...");
                    }
                    format!("Model: {} - Prompt: '{}'", model_step.model, prompt)
                }
                StepKind::Process(process_step) => format!("Process: '{}'", process_step.process),
            };
            viz_input.push_str(&format!("  - {}: {}\n", step.name, step_details));
        }

        assert!(!viz_input.contains('\n'));
        assert!(viz_input.contains("..."));
    }

    #[test]
    fn test_parse_pipeline_with_io() {
        let yaml = r#"
name: "Test I/O Pipeline"
steps:
  - name: "step-one"
    process: "cat"
    input: "input.txt"
    output: "output.txt"
  - name: "step-two"
    process: "grep -i test"
    input: "step-one"
"#;
        let pipeline: Pipeline = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(pipeline.steps[0].input, Some("input.txt".to_string()));
        assert_eq!(pipeline.steps[0].output, Some("output.txt".to_string()));
        assert_eq!(pipeline.steps[1].input, Some("step-one".to_string()));
        assert_eq!(pipeline.steps[1].output, None);
    }

use crate::execute::ExecutionArgs;

pub fn run_pipeline(base_path: &Path, pipeline: &Pipeline) -> io::Result<()> {
    use std::collections::HashMap;
    use std::process::{Command, Stdio};
    use std::io::{Read, Write};

    let mut step_outputs: HashMap<String, Vec<u8>> = HashMap::new();
    let mut previous_stdout: Option<Vec<u8>> = None;

    // Read initial input from stdin if there's something to pipe in
    let mut initial_input = Vec::new();
    let mut stdin = io::stdin();
    if let Ok(len) = stdin.read_to_end(&mut initial_input) {
        if len > 0 {
            previous_stdout = Some(initial_input);
        }
    }

    for step in &pipeline.steps {
        let mut input_data: Option<Vec<u8>> = None;

        // Determine the input for the current step
        if let Some(input_source) = &step.input {
            if Path::new(input_source).exists() {
                // Input is a file path
                input_data = Some(fs::read(input_source)?);
            } else if let Some(output) = step_outputs.get(input_source) {
                // Input is the name of a previous step
                input_data = Some(output.clone());
            }
        } else if previous_stdout.is_some() {
            // Default to the output of the previous step (piping)
            input_data = previous_stdout;
        }

        let (mut command, secrets) = match &step.kind {
            StepKind::Model(model_step) => {
                let final_prompt = if let Some(input) = &input_data {
                    // Prepend the input to the user's prompt for the model
                    format!("{}\n\n{}", String::from_utf8_lossy(input), model_step.prompt)
                } else {
                    model_step.prompt.clone()
                };

                let exec_args = ExecutionArgs {
                    model: Some(model_step.model.clone()),
                    use_context: false, // Context is explicitly managed via pipeline I/O
                    command_with_args: vec![final_prompt],
                };
                crate::execute::prepare_command(base_path, &exec_args)?
            }
            StepKind::Process(process_step) => {
                let mut cmd = Command::new("sh");
                cmd.arg("-c");
                cmd.arg(&process_step.process);
                (cmd, HashMap::new()) // Process steps don't have secrets
            }
        };

        let mut child = command
            .envs(secrets)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        // For process steps, we pipe input data directly to stdin.
        // For model steps, the input was already combined with the prompt.
        if let StepKind::Process(_) = &step.kind {
            if let Some(data) = &input_data {
                if let Some(mut child_stdin) = child.stdin.take() {
                    child_stdin.write_all(data)?;
                }
            }
        }

        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Step '{}' failed.", step.name),
            ));
        }

        // Handle output
        if let Some(output_path) = &step.output {
            fs::write(output_path, &output.stdout)?;
        }
        previous_stdout = Some(output.stdout.clone());
        step_outputs.insert(step.name.clone(), output.stdout);
    }

    if let Some(final_output) = previous_stdout {
        io::stdout().write_all(&final_output)?;
    }

    Ok(())
}

pub fn show_pipeline(pipeline: &Pipeline) -> io::Result<()> {
    use std::process::{Command, Stdio};
    use std::io::Write;

    let mut viz_input = format!("{}\n", pipeline.name);
    for step in &pipeline.steps {
        let mut step_details = match &step.kind {
            StepKind::Model(model_step) => {
                let mut prompt = model_step.prompt.replace('\n', " ");
                if prompt.len() > 40 {
                    prompt.truncate(37);
                    prompt.push_str("...");
                }
                format!("Model: {} - Prompt: '{}'", model_step.model, prompt)
            }
            StepKind::Process(process_step) => format!("Process: '{}'", process_step.process),
        };
        viz_input.push_str(&format!("  - {}: {}\n", step.name, step_details));
    }

    let mut child = Command::new("treetrunk")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(viz_input.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to execute 'treetrunk' visualization tool. Is it installed and in your PATH?",
        ));
    }

    Ok(())
}
