//! Core logic for managing the model registry.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Model {
    pub alias: String,
    pub provider: String,
    #[serde(rename = "modelName")]
    pub model_name: String,
    #[serde(rename = "apiKeySecret", skip_serializing_if = "Option::is_none")]
    pub api_key_secret: Option<String>,
    #[serde(default, skip_serializing)]
    pub discovered: bool,
}

// --- Ollama specific structs for deserialization ---
#[derive(Serialize, Deserialize, Debug)]
struct OllamaTag {
    name: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct OllamaTagsResponse {
    models: Vec<OllamaTag>,
}

/// Reads the list of registered models from the models.json file.
fn read_registered_models(base_path: &Path) -> io::Result<Vec<Model>> {
    let models_path = base_path.join(".ferri").join("models.json");
    if !models_path.exists() {
        return Ok(Vec::new());
    }
    let file_content = fs::read_to_string(models_path)?;
    let models: Vec<Model> = serde_json::from_str(&file_content)?;
    Ok(models)
}

/// Writes a list of models to the models.json file.
fn write_registered_models(base_path: &Path, models: &[Model]) -> io::Result<()> {
    let models_path = base_path.join(".ferri").join("models.json");
    let file_content = serde_json::to_string_pretty(models)?;
    fs::write(models_path, file_content)?;
    Ok(())
}

/// Adds a new model to the registry.
pub fn add_model(base_path: &Path, model: Model) -> io::Result<()> {
    let mut models = read_registered_models(base_path).unwrap_or_else(|_| Vec::new());
    // Remove any existing model with the same alias
    models.retain(|m| m.alias != model.alias);
    models.push(model);
    write_registered_models(base_path, &models)
}

/// Removes a model from the registry by its alias.
pub fn remove_model(base_path: &Path, alias: &str) -> io::Result<()> {
    let mut models = read_registered_models(base_path)?;

    // Find the model to get its details before removing it from the list
    if let Some(model_to_remove) = models.iter().find(|m| m.alias == alias).cloned() {
        // If the provider is ollama, attempt to remove the model using the ollama CLI
        if model_to_remove.provider == "ollama" {
            let status = Command::new("ollama")
                .arg("rm")
                .arg(&model_to_remove.model_name)
                .status();

            match status {
                Ok(exit_status) => {
                    if !exit_status.success() {
                        // We don't return an error here, just print a warning,
                        // as the model might not be installed locally anyway.
                        eprintln!(
                            "Warning: `ollama rm {}` failed. The model may not be installed locally.",
                            model_to_remove.model_name
                        );
                    }
                }
                Err(e) => {
                    // Ollama CLI might not be installed or in the PATH
                    eprintln!(
                        "Warning: Failed to execute `ollama rm`. Is the Ollama CLI installed and in your PATH? Error: {}",
                        e
                    );
                }
            }
        }
    }

    // Remove the model from the registry regardless of whether the CLI command succeeded
    let initial_len = models.len();
    models.retain(|m| m.alias != alias);

    if models.len() == initial_len {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Model with alias '{}' not found in the registry.", alias),
        ));
    }

    write_registered_models(base_path, &models)
}

/// Discovers local Ollama models.
fn discover_ollama_models() -> Result<Vec<Model>, reqwest::Error> {
    let url = "http://127.0.0.1:11434/api/tags";
    let response = reqwest::blocking::get(url)?;

    if !response.status().is_success() {
        // Silently fail if Ollama is not running or reachable
        return Ok(Vec::new());
    }

    let tags_response: OllamaTagsResponse = response.json()?;
    let models = tags_response
        .models
        .into_iter()
        .map(|tag| Model {
            alias: tag.name.clone(),
            provider: "ollama".to_string(),
            model_name: tag.name,
            api_key_secret: None,
            discovered: true,
        })
        .collect();
    Ok(models)
}

/// Lists all registered and discovered models.
pub fn list_models(base_path: &Path) -> io::Result<Vec<Model>> {
    let mut registered_models = read_registered_models(base_path).unwrap_or_else(|_| Vec::new());

    // Attempt to discover Ollama models, but don't fail if it's not running
    if let Ok(discovered_models) = discover_ollama_models() {
        // Add discovered models to the list, avoiding duplicates by alias
        for discovered in discovered_models {
            if !registered_models.iter().any(|r| r.alias == discovered.alias) {
                registered_models.push(discovered);
            }
        }
    }

    Ok(registered_models)
}
