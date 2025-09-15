use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn test_models_add_ls_rm() {
    // Create a temporary directory and initialize a project
    let dir = tempdir().unwrap();
    let base_path = dir.path();

    let mut cmd = Command::cargo_bin("ferri").unwrap();
    cmd.current_dir(base_path).arg("init").assert().success();

    // 1. Add a model
    let mut cmd = Command::cargo_bin("ferri").unwrap();
    cmd.current_dir(base_path)
        .arg("models")
        .arg("add")
        .arg("gpt4o")
        .arg("--provider")
        .arg("openai")
        .arg("--model-name")
        .arg("gpt-4o")
        .assert()
        .success();

    // 2. List models and verify the new model is there
    let mut cmd = Command::cargo_bin("ferri").unwrap();
    let output = cmd.current_dir(base_path).arg("models").arg("ls").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    assert!(stdout.contains("gpt4o"));
    assert!(stdout.contains("openai"));
    assert!(stdout.contains("gpt-4o"));

    // 3. Remove the model
    let mut cmd = Command::cargo_bin("ferri").unwrap();
    cmd.current_dir(base_path)
        .arg("models")
        .arg("rm")
        .arg("gpt4o")
        .assert()
        .success();

    // 4. List again and verify it's gone
    let mut cmd = Command::cargo_bin("ferri").unwrap();
    let output = cmd.current_dir(base_path).arg("models").arg("ls").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(!stdout.contains("gpt4o"));
}
