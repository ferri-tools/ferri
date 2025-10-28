use anyhow::{anyhow, Context, Result};
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time;

pub async fn generate_flow(base_path: &Path, prompt: &str) -> Result<PathBuf> {
    let api_key = env::var("GEMINI_API_KEY").map_err(|_| {
        anyhow!(
            "GEMINI_API_KEY environment variable not set.
'ferri do' requires a Google Gemini API key to generate flows.
Please set it in your environment, for example:
export GEMINI_API_KEY=\"your-api-key-here\""
        )
    })?;

    let system_prompt = r###"you are an expert software developer and terminal assistant. your goal is to break down a user's high-level request into a precise, executable ferri flow yaml file.

the user's prompt will be a high-level goal. you must convert this into a series of shell commands organized as steps within one or more jobs in a ferri flow. if the request implies a sequence of distinct stages (e.g., "build the frontend, then build the backend"), you should use multiple jobs with `needs` to define dependencies.

**key rules:**
- the output must be a valid yaml file.
- the yaml structure must conform to the `ferri.flow/v1alpha1` specification.
- use logical job ids (e.g., `build-frontend`, `deploy`).
- for multi-line commands, especially for writing files, use the yaml literal block scalar (`|`). this is the most robust method.

**example 1: simple file operations (single job)**

user prompt: "create a new directory called 'my_app' and then create an empty file inside it named 'index.js'"

your response:
```yaml
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: "create-my-app"
spec:
  jobs:
    create-structure:
      name: "Create project structure"
      runs-on: "process"
      steps:
        - name: "Create project directory"
          run: "mkdir my_app"
        - name: "Create empty index file"
          run: "touch my_app/index.js"
```

**example 2: multi-stage task (multiple jobs)**

user prompt: "create a rust project and then compile it"

your response:
```yaml
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: "create-and-build-rust-project"
spec:
  jobs:
    create-project:
      name: "Create Rust Project"
      runs-on: "process"
      steps:
        - name: "Initialize new cargo project"
          run: "cargo new my_rust_app"

    build-project:
      name: "Build Rust Project"
      runs-on: "process"
      needs:
        - create-project
      steps:
        - name: "Navigate to project and build"
          run: |
            cd my_rust_app
            cargo build
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

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .context("Failed to send request to Gemini API")?;

    if !response.status().is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        return Err(anyhow!(
            "Gemini API request failed. Body: {}",
            error_body
        ));
    }

    let response_body: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse Gemini API response")?;

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

    // Create the .ferri/do directory
    let do_dir = base_path.join(".ferri").join("do");
    fs::create_dir_all(&do_dir).context("Failed to create .ferri/do directory")?;

    // Generate a unique filename
    let timestamp = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let file_name = format!("flow-{}.yml", timestamp);
    let file_path = do_dir.join(file_name);

    // Write the flow to the file
    fs::write(&file_path, yaml_content).context("Failed to write flow to file")?;

    Ok(file_path)
}
