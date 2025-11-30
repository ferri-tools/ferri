use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_secrets_set_command() {
    let dir = tempdir().unwrap();
    let base_path = dir.path();

    // Initialize the project
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("init")
        .assert()
        .success();

    // Run the `secrets set` command
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .args(["secrets", "set", "MY_API_KEY", "123-abc-456-def"])
        .assert()
        .success();

    // Verify the secret was set by listing secrets and checking for the key
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .args(["secrets", "ls"])
        .assert()
        .success()
        .stdout(predicate::str::contains("MY_API_KEY"));
}

#[test]
fn test_secrets_ls_command() {
    let dir = tempdir().unwrap();
    let base_path = dir.path();

    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("init")
        .assert()
        .success();

    // Set a few secrets
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .args(["secrets", "set", "KEY_1", "VALUE_1"])
        .assert()
        .success();
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .args(["secrets", "set", "KEY_2", "VALUE_2"])
        .assert()
        .success();

    // Run the `secrets ls` command and check the output
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .args(["secrets", "ls"])
        .assert()
        .success()
        .stdout(predicate::str::contains("KEY_1"))
        .stdout(predicate::str::contains("KEY_2"));
}
