//! Core logic for managing encrypted secrets.

use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

// A hardcoded key for now. In a real application, this should be
// derived from a user password or stored securely.
const ENCRYPTION_KEY: &str = "a-very-secret-key-that-must-be-changed";

#[derive(Serialize, Deserialize, Debug)]
struct SecretsContainer {
    encrypted_data: String,
}

/// Sets a secret in the encrypted secrets file.
///
/// This function reads the `.ferri/secrets.json` file, decrypts the content,
/// adds or updates the secret, re-encrypts the data, and writes it back to the file.
///
/// # Arguments
///
/// * `base_path` - The root of the project, where the `.ferri` directory is located.
/// * `key` - The name of the secret to set.
/// * `value` - The value of the secret.
///
/// # Errors
///
/// Returns an error if the secrets file cannot be read, parsed, encrypted, or written.
pub fn set_secret(base_path: &Path, key: &str, value: &str) -> io::Result<()> {
    let secrets_path = base_path.join(".ferri").join("secrets.json");
    let crypt = new_magic_crypt!(ENCRYPTION_KEY, 256);

    // Read the existing file
    let file_content = fs::read_to_string(&secrets_path)?;

    let mut secrets: HashMap<String, String> = if file_content.trim().is_empty() || file_content == "{}" {
        HashMap::new()
    } else {
        let container: SecretsContainer = serde_json::from_str(&file_content)?;
        let decrypted_string = crypt.decrypt_base64_to_string(&container.encrypted_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        serde_json::from_str(&decrypted_string)?
    };

    // Insert or update the secret
    secrets.insert(key.to_string(), value.to_string());

    // Encrypt and write back
    let new_json_string = serde_json::to_string(&secrets)?;
    let encrypted_string = crypt.encrypt_str_to_base64(&new_json_string);
    let new_container = SecretsContainer { encrypted_data: encrypted_string };
    let new_file_content = serde_json::to_string_pretty(&new_container)?;

    fs::write(secrets_path, new_file_content)?;

    Ok(())
}

/// Reads and decrypts all secrets.
pub fn read_all_secrets(base_path: &Path) -> io::Result<HashMap<String, String>> {
    let secrets_path = base_path.join(".ferri").join("secrets.json");
    let crypt = new_magic_crypt!(ENCRYPTION_KEY, 256);

    let file_content = match fs::read_to_string(&secrets_path) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::initialize_project;

    // Helper function to read secrets directly for testing
    fn read_secret_for_test(base_path: &Path, key: &str) -> Option<String> {
        let secrets_path = base_path.join(".ferri").join("secrets.json");
        let crypt = new_magic_crypt!(ENCRYPTION_KEY, 256);
        let file_content = fs::read_to_string(&secrets_path).ok()?;
        if file_content.trim().is_empty() || file_content == "{}" {
            return None;
        }
        let container: SecretsContainer = serde_json::from_str(&file_content).ok()?;
        let decrypted_string = crypt.decrypt_base64_to_string(&container.encrypted_data).ok()?;
        let secrets: HashMap<String, String> = serde_json::from_str(&decrypted_string).ok()?;
        secrets.get(key).cloned()
    }

    #[test]
    fn test_set_secret_new_and_update() {
        // Setup a temporary project
        let dir = tempdir().unwrap();
        let base_path = dir.path();
        initialize_project(base_path).unwrap();

        // 1. Set a new secret
        let result = set_secret(base_path, "API_KEY", "12345");
        assert!(result.is_ok());

        // Verify the secret is set and encrypted
        let value = read_secret_for_test(base_path, "API_KEY");
        assert_eq!(value, Some("12345".to_string()));

        // 2. Update the existing secret
        let result_update = set_secret(base_path, "API_KEY", "67890");
        assert!(result_update.is_ok());

        // Verify the secret is updated
        let updated_value = read_secret_for_test(base_path, "API_KEY");
        assert_eq!(updated_value, Some("67890".to_string()));

        // 3. Add a second secret
        let result_second = set_secret(base_path, "ANOTHER_KEY", "abcde");
        assert!(result_second.is_ok());

        // Verify both secrets exist
        let first_value = read_secret_for_test(base_path, "API_KEY");
        let second_value = read_secret_for_test(base_path, "ANOTHER_KEY");
        assert_eq!(first_value, Some("67890".to_string()));
        assert_eq!(second_value, Some("abcde".to_string()));
    }
}