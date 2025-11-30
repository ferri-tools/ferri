use assert_cmd::prelude::*;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_flow_run_simple_job() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let base_path = dir.path();

    Command::cargo_bin("ferri").unwrap().current_dir(base_path).arg("init").assert().success();

    let flow_content = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: simple-job-test
spec:
  jobs:
    echo-job:
      runs_on: ubuntu-latest
      steps:
        - name: "say-hello"
          run: "echo 'Hello, World!'"
"#;
    let flow_file = base_path.join("test-flow.yml");
    fs::write(flow_file, flow_content)?;

    Command::cargo_bin("ferri")
        .unwrap()
        .current_dir(base_path)
        .args(["flow", "run", "test-flow.yml"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Flow completed successfully!"));

    Ok(())
}

#[test]
fn test_flow_run_with_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let base_path = dir.path();

    Command::cargo_bin("ferri").unwrap().current_dir(base_path).arg("init").assert().success();

    let flow_content = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: dependency-test
spec:
  jobs:
    build:
      runs_on: ubuntu-latest
      steps:
        - run: "echo 'building...'"
    test:
      runs_on: ubuntu-latest
      needs: [build]
      steps:
        - run: "echo 'testing...'"
"#;
    let flow_file = base_path.join("dep-flow.yml");
    fs::write(flow_file, flow_content)?;

    Command::cargo_bin("ferri")
        .unwrap()
        .current_dir(base_path)
        .args(["flow", "run", "dep-flow.yml"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Flow completed successfully!"));

    Ok(())
}

#[test]
fn test_flow_run_with_failing_job() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let base_path = dir.path();

    Command::cargo_bin("ferri").unwrap().current_dir(base_path).arg("init").assert().success();

    let flow_content = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: failing-job-test
spec:
  jobs:
    bad-job:
      runs_on: ubuntu-latest
      steps:
        - name: "exit-one"
          run: "exit 1"
"#;
    let flow_file = base_path.join("fail-flow.yml");
    fs::write(flow_file, flow_content)?;

    Command::cargo_bin("ferri")
        .unwrap()
        .current_dir(base_path)
        .args(["flow", "run", "fail-flow.yml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Flow execution failed"));

    Ok(())
}

#[test]
fn test_flow_show() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let base_path = dir.path();

    Command::cargo_bin("ferri").unwrap().current_dir(base_path).arg("init").assert().success();

    let flow_content = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: show-test
spec:
  jobs:
    job-a:
      runs_on: ubuntu-latest
      steps:
        - run: "echo a"
    job-b:
      runs_on: ubuntu-latest
      needs: [job-a]
      steps:
        - run: "echo b"
"#;
    let flow_file = base_path.join("show-flow.yml");
    fs::write(flow_file, flow_content)?;

    Command::cargo_bin("ferri")
        .unwrap()
        .current_dir(base_path)
        .args(["flow", "show", "show-flow.yml"])
        .write_stdin("q\n")
        .assert()
        .success();

    Ok(())
}
