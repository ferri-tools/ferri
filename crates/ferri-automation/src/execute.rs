//! Core logic for executing commands with injected context.

use ferri_core::{context, models, secrets};
use clap::Args;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

#[derive(Args, Debug, Clone)]
pub struct SharedArgs {
    /// The model to use for the command
    #[arg(long)]
    pub model: Option<String>,
    /// Inject context into the command
    #[arg(long)]
    pub ctx: bool,
    /// The file path to save the output to
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Force streaming output
    #[arg(long)]
    pub stream: bool,
    /// The command to execute
    #[arg(required = true, trailing_var_arg = true)]
    pub command: Vec<String>,
}

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
    GoogleGeminiImage,
    Unknown,
}

/// Arguments for preparing a command for execution.
pub struct ExecutionArgs {
    pub model: Option<String>,
    pub use_context: bool,
    pub output_file: Option<PathBuf>,
    pub command_with_args: Vec<String>,
    pub streaming: bool,
}

// A unified return type for command execution
pub enum PreparedCommand {
    Local(Command, Option<String>),
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
            "google-gemini-image" => ModelProvider::GoogleGeminiImage,
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
                let final_prompt = if args.use_context {
                    let full_context = context::get_full_context(base_path)?;
                    format!(
                        "{}

Use the content of the files below as context to answer the question.

{}",
                        prompt,
                        full_context.trim()
                    )
                } else {
                    prompt
                };
                let mut command = Command::new("ollama");
                command.arg("run").arg(&model.model_name);
                Ok((PreparedCommand::Local(command, Some(final_prompt)), decrypted_secrets))
            }
            ModelProvider::Google => {
                let api_key = api_key.ok_or_else(|| Error::new(ErrorKind::NotFound, "Google provider requires an API key secret."))?;
                let endpoint = if args.streaming { "streamGenerateContent" } else { "generateContent" };
                let url = format!("https://generativelanguage.googleapis.com/v1/models/{}:{}", model.model_name, endpoint);

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
                let request = client.post(&url)
                    .header("x-goog-api-key", api_key)
                    .header("Content-Type", "application/json")
                    .json(&body);
                Ok((PreparedCommand::Remote(request), decrypted_secrets))
            }
            ModelProvider::GoogleGeminiImage => {
                let api_key = api_key.ok_or_else(|| Error::new(ErrorKind::NotFound, "Google provider requires an API key secret."))?;
                let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model.model_name);
                let body = json!({ "contents": [{ "parts": [{ "text": prompt }] }] });
                let client = reqwest::blocking::Client::new();
                let request = client.post(&url)
                    .header("x-goog-api-key", api_key)
                    .header("Content-Type", "application/json")
                    .json(&body);
                Ok((PreparedCommand::Remote(request), decrypted_secrets))
            }
            ModelProvider::Unknown => {
                return Err(Error::new(ErrorKind::InvalidInput, format!("Unknown model provider: {}", model.provider)));
            }
        }
    } else {
        let mut final_command_with_args = final_command_with_args;
        if args.use_context {
            let context = context::get_full_context(base_path)?;
            if let Some(last_arg) = final_command_with_args.last_mut() {
                *last_arg = format!("{}\n{}", context, last_arg);
            }
        }
        let command_name = &final_command_with_args[0];
        let command_args = &final_command_with_args[1..];
        let mut command = Command::new(command_name);
        command.args(command_args);
        Ok((PreparedCommand::Local(command, None), decrypted_secrets))
    }
}

/// Decodes a base64 string and saves it as an image file.
pub fn save_base64_image(path: &Path, b64_data: &str) -> io::Result<()> {
    let image_bytes = STANDARD.decode(b64_data)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("Failed to decode base64: {}", e)))?;
    fs::write(path, image_bytes)
}

/// Prepares and executes a streaming request to the Google Gemini API.
/// This function is self-contained and only used by the `ferri with` command.
pub async fn execute_streaming_gemini_request(
    base_path: &Path,
    args: &ExecutionArgs,
) -> io::Result<reqwest::RequestBuilder> {
    let model_alias = args.model.as_ref().ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Model alias is required for streaming."))?;

    let all_models = models::list_models(base_path)?;
    let model = all_models.iter().find(|m| m.alias == *model_alias)
        .ok_or_else(|| Error::new(ErrorKind::NotFound, format!("Model '{}' not found.", model_alias)))?;

    let api_key = if let Some(secret_name) = &model.api_key_secret {
        secrets::read_secret(base_path, secret_name)?
    } else {
        return Err(Error::new(ErrorKind::NotFound, "Google provider requires an API key secret."));
    };

    let prompt = args.command_with_args.join(" ");
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent", model.model_name);

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

    let client = reqwest::Client::new();
    let request = client.post(&url)
        .header("x-goog-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&body);

    Ok(request)
}