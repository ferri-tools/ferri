use assert_cmd::prelude::*;
use predicates::prelude::*;
use assert_cmd::Command;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_flow_run_simple_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let base_path = dir.path();

    // 1. Initialize project
    let mut cmd = Command::cargo_bin("ferri")?;
    cmd.current_dir(base_path).arg("init").assert().success();

    // 2. Create a simple flow.yml file
    let flow_content = r#"
name: "Test Flow"
steps:
  - name: "add-greeting"
    process: "sed 's/^/Hello, /'"
  - name: "add-punctuation"
    process: "sed 's/$/!/'"
"#;
    let flow_file = base_path.join("test-flow.yml");
    fs::write(flow_file, flow_content)?;

    // 3. Run `ferri flow run` with piped input
    let mut flow_cmd = Command::cargo_bin("ferri")?;
    flow_cmd
        .current_dir(base_path)
        .args(["flow", "run", "test-flow.yml"])
        .write_stdin("World")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, World!"));

    Ok(())
}

#[test]
fn test_flow_run_with_io() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let base_path = dir.path();

    // 1. Initialize project
    let mut cmd = Command::cargo_bin("ferri")?;
    cmd.current_dir(base_path).arg("init").assert().success();

    // 2. Create an input file
    let input_file = base_path.join("input.txt");
    fs::write(&input_file, "Line 1\nTest Line 2\nLine 3")?;

    // 3. Create a flow.yml file with I/O
    let flow_content = r#"##"
name: "Test I/O Flow"
steps:
  - name: "read-file"
    process: "cat"
    input: "input.txt"
  - name: "grep-step"
    process: "grep Test"
    input: "read-file"
    output: "output.txt"
##"#;
    let flow_file = base_path.join("io-flow.yml");
    fs::write(flow_file, flow_content)?;

    // 4. Run the flow
    let mut flow_cmd = Command::cargo_bin("ferri")?;
    flow_cmd
        .current_dir(base_path)
        .args(["flow", "run", "io-flow.yml"])
        .assert()
        .success();

    // 5. Assert the output file has the correct content
    let output_file = base_path.join("output.txt");
    let output_content = fs::read_to_string(output_file)?;
    assert_eq!(output_content.trim(), "Test Line 2");

    Ok(())
}

#[test]
fn test_flow_run_with_model_and_secret() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let base_path = dir.path();

    // 1. Initialize project, set secret, and add model
    Command::cargo_bin("ferri")?.current_dir(base_path).arg("init").assert().success();
    Command::cargo_bin("ferri")?.current_dir(base_path).args(["secrets", "set", "TEST_KEY", "secret_value"]).assert().success();
    Command::cargo_bin("ferri")?.current_dir(base_path).args([
        "models", "add", "test-model",
        "--provider", "openai", // Provider doesn't matter, just needs a secret
        "--model-name", "test-name",
        "--api-key-secret", "TEST_KEY",
    ]).assert().success();

    // 2. Create a flow.yml that uses the model
    // We will have the model step echo the secret to verify it was injected.
    let flow_content = r#"
name: "Test Model Flow"
steps:
  - name: "run-model"
    model:
      model: "test-model"
      prompt: "The secret is $TEST_KEY" # This prompt will be executed by `sh -c`
  - name: "check-output"
    process: "grep secret_value"
"#;
    let flow_file = base_path.join("model-flow.yml");
    fs::write(&flow_file, flow_content)?;

    // 3. Run the flow
    // The `prepare_command` will turn the model step into `sh -c 'echo The secret is $TEST_KEY'`
    // after it prepends the `ollama run` or similar command. For this test, we can
    // rely on the fact that the secret is injected as an env var.
    let mut flow_cmd = Command::cargo_bin("ferri")?;
    flow_cmd
        .current_dir(base_path)
        .args(["flow", "run", "model-flow.yml"])
        .assert()
        .success()
        .stdout(predicate::str::contains("secret_value"));

    Ok(())
}

#[test]
fn test_flow_show() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let base_path = dir.path();

    // 1. Initialize project
    Command::cargo_bin("ferri")?.current_dir(base_path).arg("init").assert().success();

    // 2. Create a flow file
    let flow_content = r#"
name: "Show Test Flow"
steps:
  - name: "step-a"
    process: "echo 'a'"
  - name: "step-b"
    model:
      model: "gemma"
      prompt: "b"
"#;
    let flow_file = base_path.join("show-flow.yml");
    fs::write(flow_file, flow_content)?;

    // 3. Run `ferri flow show` and pipe 'q' to stdin to quit the TUI
    let mut flow_cmd = Command::cargo_bin("ferri")?;
    flow_cmd
        .current_dir(base_path)
        .args(["flow", "show", "show-flow.yml"])
        .write_stdin("q")
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_flow_run_shell_injection_vulnerability_fixed() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let base_path = dir.path();

    // 1. Initialize project
    Command::cargo_bin("ferri")?.current_dir(base_path).arg("init").assert().success();

    // 2. Create a malicious flow.yml file
    // This command attempts to create a file named 'exploit'.
    let malicious_command = "touch exploit";
    let flow_content = format!(r#"
name: "Malicious Flow"
steps:
  - name: "exploit-step"
    process: "echo hello; {}"
"#, malicious_command);
    let flow_file = base_path.join("malicious-flow.yml");
    fs::write(flow_file, flow_content)?;

    // 3. Run the flow
    let mut flow_cmd = Command::cargo_bin("ferri")?;
    flow_cmd
        .current_dir(base_path)
        .args(["flow", "run", "malicious-flow.yml"])
        .assert()
        .success();

    // 4. Assert that the exploit file was NOT created
    let exploit_file = base_path.join("exploit");
    assert!(!exploit_file.exists(), "Vulnerability NOT fixed: Exploit file was created!");

    Ok(())
}
