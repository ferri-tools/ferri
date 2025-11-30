use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_models_add_ls_rm_confirm_yes() {
    let dir = tempdir().unwrap();
    let base_path = dir.path();

    // Init project
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("init")
        .assert()
        .success();

    // 1. Add an ollama model
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .args([
            "models",
            "add",
            "test-llama",
            "--provider",
            "ollama",
            "--model-name",
            "llama3:test",
        ])
        .assert()
        .success();

    // 2. List models and verify it's there
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .args(["models", "ls"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-llama"))
        .stdout(predicate::str::contains("ollama"))
        .stdout(predicate::str::contains("llama3:test"));

    // 3. Remove the model, confirming with "y"
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .args(["models", "rm", "test-llama"])
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Model 'test-llama' removed successfully."));

    // 4. List again and verify it's gone
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .args(["models", "ls"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-llama").not());
}

#[test]
fn test_models_rm_confirm_no() {
    let dir = tempdir().unwrap();
    let base_path = dir.path();

    // Init project
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("init")
        .assert()
        .success();

    // 1. Add a model
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .args([
            "models",
            "add",
            "test-model",
            "--provider",
            "some-provider",
            "--model-name",
            "some-model",
        ])
        .assert()
        .success();

    // 2. Attempt to remove the model, but cancel
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .args(["models", "rm", "test-model"])
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Removal cancelled."));

    // 3. List again and verify it's still there
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .args(["models", "ls"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-model"));
}
