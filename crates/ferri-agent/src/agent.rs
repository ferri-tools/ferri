use ferri_automation::execute::PreparedCommand;
use ferri_automation::jobs;
use anyhow::{anyhow, Context, Result};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

pub async fn generate_and_run_flow<F>(
    base_path: &Path,
    prompt: &str,
    mut status_callback: F,
) -> Result<String>
where
    F: FnMut(&str),
{
    status_callback(&format!("Generating flow for prompt: '{}'", prompt));

    let api_key =
        env::var("GEMINI_API_KEY").context("GEMINI_API_KEY environment variable not set")?;

    let system_prompt = r###"you are an expert software developer and terminal assistant. your goal is to break down a user's high-level request into a precise, executable ferri flow yaml file.

the user's prompt will be a high-level goal. you must convert this into a series of shell commands organized as steps in a ferri flow.

**key rules:**
- the output must be a valid yaml file.
- the yaml structure must have a top-level `name` and a list of `steps`.
- each item in the `steps` list is an object with a `name` (a human-readable title) and a `command` (the shell command to execute).
- **important**: for multi-line commands, especially for writing files, use the yaml literal block scalar (`|`). this is the most robust method.
- steps are executed sequentially. do not use a `dependencies` field.

**example 1: simple file operations**

user prompt: "create a new directory called 'my_app' and then create an empty file inside it named 'index.js'"

your response:
```yaml
name: "create my_app directory and index.js file"
steps:
  - name: "create project directory"
    command: "mkdir my_app"
  - name: "create empty index file"
    command: "touch my_app/index.js"
```

**example 2: writing a multi-line script file**

user prompt: "create a python script named 'app.py' that prints 'hello'"

your response:
```yaml
name: "create hello world python script"
steps:
  - name: "write python script"
    command: |
      cat > app.py << EOF
      #!/usr/bin/env python3
      print('hello')
      EOF
```
"###;

    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-pro-latest:generateContent?key={}",
        api_key
    );

    let request_body = json!({
        "contents": [
            {
                "parts": [
                    { "text": system_prompt },
                    { "text": prompt }
                ]
            }
        ]
    });

    status_callback("Sending request to Gemini API...");

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .context("Failed to send request to Gemini API")?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        return Err(anyhow!(
            "Gemini API request failed with status: {}. Body: {}",
            status,
            error_body
        ));
    }

    let response_body: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse Gemini API response")?;

    status_callback("Received response from Gemini API. Parsing flow...");

    let generated_text = response_body["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .context("Could not extract generated text from Gemini API response")?;

    let yaml_content = if let Some(start_index) = generated_text.find("```yaml") {
        let after_start = &generated_text[start_index + 7..];
        if let Some(end_index) = after_start.find("```") {
            &after_start[..end_index]
        } else {
            after_start
        }
    } else {
        generated_text
    };

    let yaml_content = yaml_content.trim();

    status_callback(&format!(
        "Generated Flow:\n---\n{}\n---",
        yaml_content
    ));

    let mut temp_file = NamedTempFile::new().context("Failed to create temporary file")?;
    writeln!(temp_file, "{}", yaml_content).context("Failed to write to temporary file")?;

    status_callback("Executing generated flow...");

    let command_str = format!("ferri flow run {}", temp_file.path().to_string_lossy());
    let mut command = Command::new("sh");
    command.arg("-c").arg(&command_str);

    let job = jobs::submit_job(
        base_path,
        PreparedCommand::Local(command, None),
        HashMap::new(),
        &[],
        None,
        None,
    )?;

    temp_file.keep()?;

    Ok(job.id)
}
