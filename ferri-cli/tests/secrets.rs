use assert_cmd::Command;
use tempfile::tempdir;

// Helper function to read secrets directly for testing
// NOTE: This is duplicated from core logic tests. In a real project,
// you might share this via a test-utils crate.
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const ENCRYPTION_KEY: &str = "a-very-secret-key-that-must-be-changed";

#[derive(Deserialize, Debug)]
struct SecretsContainer {
    encrypted_data: String,
}

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
fn test_secrets_set_command() {
    // Create a temporary directory and initialize a project
    let dir = tempdir().unwrap();
    let base_path = dir.path();

    let mut cmd = Command::cargo_bin("ferri-cli").unwrap();
    cmd.current_dir(base_path).arg("init").assert().success();

    // Run the `secrets set` command
    let mut cmd = Command::cargo_bin("ferri-cli").unwrap();
    cmd.current_dir(base_path)
        .arg("secrets")
        .arg("set")
        .arg("MY_API_KEY")
        .arg("123-abc-456-def")
        .assert()
        .success();

    // Verify the secret was set correctly by reading it back
    let value = read_secret_for_test(base_path, "MY_API_KEY");
    assert_eq!(value, Some("123-abc-456-def".to_string()));
}

#[test]
fn test_secrets_ls_command() {
    // Create a temporary directory and initialize a project
    let dir = tempdir().unwrap();
    let base_path = dir.path();

    let mut cmd = Command::cargo_bin("ferri-cli").unwrap();
    cmd.current_dir(base_path).arg("init").assert().success();

    // Set a few secrets
    let mut cmd = Command::cargo_bin("ferri-cli").unwrap();
    cmd.current_dir(base_path).args(&["secrets", "set", "KEY_1", "VALUE_1"]).assert().success();
    let mut cmd = Command::cargo_bin("ferri-cli").unwrap();
    cmd.current_dir(base_path).args(&["secrets", "set", "KEY_2", "VALUE_2"]).assert().success();

    // Run the `secrets ls` command
    let mut cmd = Command::cargo_bin("ferri-cli").unwrap();
    let assert = cmd.current_dir(base_path).args(&["secrets", "ls"]).assert().success();

    // Check the output
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(output.contains("KEY_1"));
    assert!(output.contains("KEY_2"));
}
