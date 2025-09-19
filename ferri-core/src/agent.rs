use crate::flow::{parse_pipeline_file, run_pipeline_plain};
use anyhow::{anyhow, Context, Result};
use serde_json::json;
use std::env;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

pub async fn generate_and_run_flow(base_path: &Path, prompt: &str) -> Result<()> {
    println!("[AGENT] Generating flow for prompt: '{}'", prompt);

    let api_key =
        env::var("GEMINI_API_KEY").context("GEMINI_API_KEY environment variable not set")?;

    // Corrected the system prompt to generate a nested `process` map to match the `ProcessStep` struct.
    let system_prompt = r#"you are an expert software developer and terminal assistant. your goal is to break down a user's high-level request into a precise, executable ferri flow yaml file.

the user's prompt will be a high-level goal. you must convert this into a series of shell commands organized as steps in a ferri flow.

**key rules:**
- the output must be a valid yaml file.
- the yaml structure must have a top-level `name` and a list of `steps`.
- each item in the `steps` list is an object with a `name` (a human-readable title).
- to define a command, use the `process` key. the value of this key must be another map that contains a `process` key with the shell command as its value.
- steps are executed sequentially. do not use a `dependencies` field.

**example 1: simple file operations**

user prompt: "create a new directory called 'my_app' and then create an empty file inside it named 'index.js'"

your response:
```yaml
name: "create my_app directory and index.js file"
steps:
  - name: "create project directory"
    process:
      process: "mkdir my_app"
  - name: "create empty index file"
    process:
      process: "touch my_app/index.js"
```

**example 2: git operations**

user prompt: "create a new feature branch called 'new-login-flow', stage all current changes, and then commit them with the message 'feat: start login flow'"

your response:
```yaml
name: "create and commit to new feature branch"
steps:
  - name: "create new feature branch"
    process:
      process: "git checkout -b new-login-flow"
  - name: "stage all changes"
    process:
      process: "git add ."
  - name: "commit changes"
    process:
      process: "git commit -m 'feat: start login flow'"
```
"#;

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

    println!("[AGENT] Sending request to Gemini API...");

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

    println!("[AGENT] Received response from Gemini API.");

    let generated_text = response_body["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .context("Could not extract generated text from Gemini API response")?;

    let yaml_content = generated_text
        .trim()
        .strip_prefix("```yaml")
        .unwrap_or(generated_text)
        .strip_suffix("```")
        .unwrap_or(generated_text)
        .trim();

    println!(
        "\n--- Generated Flow ---\n{}\n----------------------\n",
        yaml_content
    );

    let mut temp_file = NamedTempFile::new().context("Failed to create temporary file")?;
    writeln!(temp_file, "{}", yaml_content).context("Failed to write to temporary file")?;

    let pipeline = parse_pipeline_file(temp_file.path())?;

    run_pipeline_plain(base_path, &pipeline).await
}
