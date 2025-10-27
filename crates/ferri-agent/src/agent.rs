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

    let api_key = env::var("GEMINI_API_KEY").map_err(|_| {
        anyhow!(
            "GEMINI_API_KEY environment variable not set.
'ferri do' requires a Google Gemini API key to generate flows.
Please set it in your environment, for example:
export GEMINI_API_KEY=\"your-api-key-here\""
        )
    })?;

    let system_prompt = r###"you are an expert software developer and terminal assistant. your goal is to break down a user's high-level request into a precise, executable ferri flow yaml file.

the user's prompt will be a high-level goal. you must convert this into a series of shell commands organized as steps within a single job in a ferri flow.

**key rules:**
- the output must be a valid yaml file.
- the yaml structure must conform to the `ferri.flow/v1alpha1` specification.
- create a single job with a logical id (e.g., `main-job`).
- all commands should be placed as steps within that single job.
- **important**: for multi-line commands, especially for writing files, use the yaml literal block scalar (`|`). this is the most robust method.

**example 1: simple file operations**

user prompt: "create a new directory called 'my_app' and then create an empty file inside it named 'index.js'"

your response:
```yaml
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: "create-my-app"
spec:
  jobs:
    main-job:
      name: "Create project structure"
      runs-on: "local"
      steps:
        - name: "Create project directory"
          run: "mkdir my_app"
        - name: "Create empty index file"
          run: "touch my_app/index.js"
```

**example 2: writing a multi-line script file**

user prompt: "create a python script named 'app.py' that prints 'hello'"

your response:
```yaml
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: "create-python-script"
spec:
  jobs:
    main-job:
      name: "Create Hello World Python Script"
      runs-on: "local"
      steps:
        - name: "Write python script"
          run: |
            cat > app.py << EOF
            #!/usr/bin/env python3
            print('hello')
            EOF
```
"###;

    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-pro:generateContent?key={}",
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

    // Create the .ferri/do directory
    let do_dir = base_path.join(".ferri").join("do");
    std::fs::create_dir_all(&do_dir).context("Failed to create .ferri/do directory")?;

    // Generate a unique filename
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let file_name = format!("flow-{}.yml", timestamp);
    let file_path = do_dir.join(file_name);

    // Write the flow to the file
    std::fs::write(&file_path, yaml_content).context("Failed to write flow to file")?;

    status_callback(&format!(
        "Executing generated flow: {}",
        file_path.to_string_lossy()
    ));

    let base_path_buf = base_path.to_path_buf();
    let job = tokio::task::spawn_blocking(move || {
        let command_str = format!("ferri flow run --quiet {}", file_path.to_string_lossy());
        let mut command = Command::new("sh");
        command.arg("-c").arg(&command_str);

        jobs::submit_job(
            &base_path_buf,
            PreparedCommand::Local(command, None),
            HashMap::new(),
            &[],
            None,
            None,
        )
    })
    .await
    .context("Failed to spawn blocking task for job submission")?
    .context("Job submission failed in blocking task")?;

    Ok(job.id)
}
