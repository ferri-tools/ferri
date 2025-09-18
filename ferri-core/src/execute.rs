//! Core logic for executing commands with injected context.

use crate::{context, models, secrets};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::io::{self, Error, ErrorKind};
use std::path::Path;
use std::process::Command;
use std::fs;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

// --- Structs for deserializing Gemini API responses ---
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Candidate {
    content: Content,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Content {
    parts: Vec<Part>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Part {
    text: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct GoogleApiErrorResponse {
    error: GoogleApiError,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct GoogleApiError {
    message: String,
}
// ---

// Enum to represent different model providers
pub enum ModelProvider {
    Ollama,
    Google,
    Unknown,
}

/// Arguments for preparing a command for execution.
pub struct ExecutionArgs {
    pub model: Option<String>,
    pub use_context: bool,
    pub command_with_args: Vec<String>,
}

// A unified return type for command execution
pub enum PreparedCommand {
    Local(Command),
    Remote(reqwest::blocking::RequestBuilder),
}

/// Prepares a command or an API request.
pub fn prepare_command(
    base_path: &Path,
    args: &ExecutionArgs,
) -> io::Result<(PreparedCommand, HashMap<String, String>)> {
    let mut decrypted_secrets = secrets::read_all_secrets(base_path)?;
    let final_command_with_args = args.command_with_args.clone();

    if let Some(model_alias) = &args.model {
        let all_models = models::list_models(base_path)?;
        let model = all_models.iter().find(|m| m.alias == *model_alias)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, format!("Model '{}' not found.", model_alias)))?;

        let provider = match model.provider.as_str() {
            "ollama" => ModelProvider::Ollama,
            "google" => ModelProvider::Google,
            _ => ModelProvider::Unknown,
        };

        let api_key = if let Some(secret_name) = &model.api_key_secret {
            let key = secrets::read_secret(base_path, secret_name)?;
            decrypted_secrets.insert(secret_name.clone(), key.clone());
            Some(key)
        } else {
            None
        };

        let prompt = final_command_with_args.join(" ");

        match provider {
            ModelProvider::Ollama => {
                let mut command = Command::new("ollama");
                let final_prompt = if args.use_context {
                    let full_context = context::get_full_multimodal_context(base_path)?;
                    // NOTE: Ollama doesn't support multimodal input in the same way as Gemini yet.
                    // We will only use the text context for now.
                    format!(
                        "You are a helpful assistant. Use the following file content to answer the user's question.\n\n---\n{}\n---\n\nQuestion: {}",
                        full_context.text_content.trim(),
                        prompt
                    )
                } else {
                    prompt
                };
                command.arg("run").arg(&model.model_name).arg(final_prompt);
                Ok((PreparedCommand::Local(command), decrypted_secrets))
            }
            ModelProvider::Google => {
                let api_key = api_key.ok_or_else(|| Error::new(ErrorKind::NotFound, "Google provider requires an API key secret."))?;
                let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?key={}", model.model_name, api_key);

                let mut parts = Vec::new();

                if args.use_context {
                    let full_context = context::get_full_multimodal_context(base_path)?;
                    let context_prompt = format!(
                        "You are a helpful assistant. Use the following file content to answer the user's question.\n\n---\n{}\n---\n\nQuestion: {}",
                        full_context.text_content.trim(),
                        prompt
                    );
                    parts.push(json!({ "text": context_prompt }));

                    for image_file in full_context.image_files {
                        let image_data = fs::read(&image_file.path)?;
                        let encoded_image = STANDARD.encode(image_data);
                        let mime_type = match image_file.content_type {
                            context::ContentType::Png => "image/png",
                            context::ContentType::Jpeg => "image/jpeg",
                            context::ContentType::WebP => "image/webp",
                            _ => "application/octet-stream",
                        };
                        parts.push(json!({
                            "inline_data": {
                                "mime_type": mime_type,
                                "data": encoded_image
                            }
                        }));
                    }
                } else {
                    parts.push(json!({ "text": prompt }));
                }

                let body = json!({ "contents": [{ "parts": parts }] });

                let client = reqwest::blocking::Client::new();
                let request = client.post(&url).json(&body);
                Ok((PreparedCommand::Remote(request), decrypted_secrets))
            }
            ModelProvider::Unknown => {
                let command_name = &final_command_with_args[0];
                let command_args = &final_command_with_args[1..];
                let mut command = Command::new(command_name);
                command.args(command_args);
                Ok((PreparedCommand::Local(command), decrypted_secrets))
            }
        }
    } else {
        let command_name = &final_command_with_args[0];
        let command_args = &final_command_with_args[1..];
        let mut command = Command::new(command_name);
        command.args(command_args);
        Ok((PreparedCommand::Local(command), decrypted_secrets))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{initialize_project, models, secrets};
    use tempfile::tempdir;

    #[test]
    fn test_prepare_google_model_request() {
        let dir = tempdir().unwrap();
        let base_path = dir.path();
        initialize_project(base_path).unwrap();
        secrets::set_secret(base_path, "GOOGLE_API_KEY", "test-key").unwrap();
        let model = models::Model {
            alias: "gemini".to_string(),
            provider: "google".to_string(),
            model_name: "gemini-pro".to_string(),
            api_key_secret: Some("GOOGLE_API_KEY".to_string()),
            discovered: false,
        };
        models::add_model(base_path, model).unwrap();

        let args = ExecutionArgs {
            model: Some("gemini".to_string()),
            use_context: false,
            command_with_args: vec!["hello".to_string()],
        };

        let result = prepare_command(base_path, &args);
        assert!(result.is_ok());
        let (prepared, _) = result.unwrap();
        match prepared {
            PreparedCommand::Remote(req) => {
                let req = req.build().unwrap();
                assert_eq!(req.method(), "POST");
                assert!(req.url().as_str().contains("gemini-pro:generateContent?key=test-key"));
                let body_bytes = req.body().unwrap().as_bytes().unwrap();
                let body_json: serde_json::Value = serde_json::from_slice(body_bytes).unwrap();
                assert_eq!(body_json["contents"][0]["parts"][0]["text"], "hello");
            }
            _ => panic!("Expected a remote command"),
        }
    }
}
