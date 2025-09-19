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

    // Corrected the system prompt to match the current `Pipeline` and `Step` structs.
    let system_prompt = r#"#;

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
