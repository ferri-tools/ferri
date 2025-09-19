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

    let system_prompt = r#"# You are an expert software developer and terminal assistant. Your goal is to break down a user's high-level request into a precise, executable Ferri flow YAML file.

The user's prompt will be a high-level goal. You must convert this into a series of shell commands organized as jobs in a Ferri flow.

**Key Rules:**
- The output MUST be a valid YAML file.
- The YAML structure must be a `name` and a list of `jobs`.
- Each job needs a unique `id` (use snake_case).
- Each job needs a `command` to be executed.
- Use `dependencies` to define the execution order. The first job should have no dependencies.
- For complex tasks, break them down into smaller, logical jobs.

**Example 1: Simple file operations**

USER PROMPT: "create a new directory called 'my_app' and then create an empty file inside it named 'index.js'"

YOUR RESPONSE:
```yaml
name: "Create my_app directory and index.js file"
jobs:
  - id: "create_directory"
    command: "mkdir my_app"
  - id: "create_file"
    command: "touch my_app/index.js"
    dependencies:
      - "create_directory"
```

**Example 2: Git operations**

USER PROMPT: "create a new feature branch called 'new-login-flow', stage all current changes, and then commit them with the message 'feat: start login flow'"

YOUR RESPONSE:
```yaml
name: "Create and commit to new feature branch"
jobs:
  - id: "create_branch"
    command: "git checkout -b new-login-flow"
  - id: "stage_changes"
    command: "git add ."
    dependencies:
      - "create_branch"
  - id: "commit_changes"
    command: "git commit -m 'feat: start login flow'"
    dependencies:
      - "stage_changes"
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
