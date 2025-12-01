//! Core logic for managing encrypted secrets.

use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use rpassword::prompt_password;

// A hardcoded key for now. In a real application, this should be
// derived from a user password or stored securely.
const ENCRYPTION_KEY: &str = "a-very-secret-key-that-must-be-changed";

#[derive(Serialize, Deserialize, Debug)]
struct SecretsContainer {
    encrypted_data: String,
}

/// Sets a secret. If a value is provided, it's used directly. Otherwise, prompts interactively.
pub fn set_secret(base_path: &Path, key: &str, value: Option<String>) -> io::Result<()> {
    let secrets_path = base_path.join(".ferri").join("secrets.json");
    let crypt = new_magic_crypt!(ENCRYPTION_KEY, 256);

    // Use the provided value or prompt for it interactively
    let final_value = match value {
        Some(v) => v,
        None => prompt_password(format!("Enter value for '{}': ", key))?,
    };

    let mut secrets = read_all_secrets_internal(base_path, &secrets_path, &crypt)?;

    // Insert or update the secret
    secrets.insert(key.to_string(), final_value);

    // Encrypt and write back
    write_all_secrets_internal(&secrets_path, &crypt, &secrets)?;

    println!("Secret '{}' set successfully.", key);
    Ok(())
}

/// Removes a secret from the encrypted secrets file.
pub fn remove_secret(base_path: &Path, key: &str) -> io::Result<()> {
    let secrets_path = base_path.join(".ferri").join("secrets.json");
    let crypt = new_magic_crypt!(ENCRYPTION_KEY, 256);

    let mut secrets = read_all_secrets_internal(base_path, &secrets_path, &crypt)?;

    // Remove the secret
    if secrets.remove(key).is_none() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Secret '{}' not found.", key)));
    }

    // Encrypt and write back
    write_all_secrets_internal(&secrets_path, &crypt, &secrets)?;

    Ok(())
}

/// Reads and decrypts all secrets.
pub fn read_all_secrets(base_path: &Path) -> io::Result<HashMap<String, String>> {
    let secrets_path = base_path.join(".ferri").join("secrets.json");
    let crypt = new_magic_crypt!(ENCRYPTION_KEY, 256);
    read_all_secrets_internal(base_path, &secrets_path, &crypt)
}

/// Lists the keys of all stored secrets.
pub fn list_secrets(base_path: &Path) -> io::Result<Vec<String>> {
    let secrets = read_all_secrets(base_path)?;
    let mut keys: Vec<String> = secrets.keys().cloned().collect();
    keys.sort();
    Ok(keys)
}

/// Reads and decrypts a single secret by its key.
pub fn read_secret(base_path: &Path, key: &str) -> io::Result<String> {
    let secrets = read_all_secrets(base_path)?;
    secrets
        .get(key)
        .cloned()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("Secret '{}' not found.", key)))
}

// --- Internal Helper Functions ---

fn read_all_secrets_internal<M: MagicCryptTrait>(
    _base_path: &Path,
    secrets_path: &Path,
    crypt: &M,
) -> io::Result<HashMap<String, String>> {
    let file_content = match fs::read_to_string(secrets_path) {
        Ok(content) => content,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(e) => return Err(e),
    };

    if file_content.trim().is_empty() || file_content == "{}" {
        return Ok(HashMap::new());
    }

    let container: SecretsContainer = serde_json::from_str(&file_content)?;
    let decrypted_string = crypt.decrypt_base64_to_string(&container.encrypted_data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    serde_json::from_str(&decrypted_string)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn write_all_secrets_internal<M: MagicCryptTrait>(
    secrets_path: &Path,
    crypt: &M,
    secrets: &HashMap<String, String>,
) -> io::Result<()> {
    let new_json_string = serde_json::to_string(secrets)?;
    let encrypted_string = crypt.encrypt_str_to_base64(&new_json_string);
    let new_container = SecretsContainer { encrypted_data: encrypted_string };
    let new_file_content = serde_json::to_string_pretty(&new_container)?;

    fs::write(secrets_path, new_file_content)
}