use assert_cmd::prelude::*;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_with_and_run_unification() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let base_path = dir.path();

    // 1. Initialize project
    Command::cargo_bin("ferri").unwrap().current_dir(base_path).arg("init").assert().success();

    // 2. Set a secret
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .args(["secrets", "set", "MY_API_KEY", "test-key"])
        .assert()
        .success();

    // 3. Test `ferri with` using a simple command
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("with")
        .arg("--")
        .arg("printenv")
        .arg("MY_API_KEY")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-key"));

    // 4. Test `ferri run`, `ps`, and `yank`
    let run_output = Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("run")
        .arg("--")
        .arg("sh")
        .arg("-c")
        .arg("echo $MY_API_KEY && echo 'background job'")
        .output()?;

    assert!(run_output.status.success());
    let stdout = String::from_utf8(run_output.stdout)?;
    let job_id = stdout
        .lines()
        .find(|line| line.contains("Successfully submitted job"))
        .and_then(|line| line.split('\'').nth(1))
        .ok_or("Could not parse job ID")?;

    // Give the job a moment to complete
    std::thread::sleep(std::time::Duration::from_millis(200));

    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("ps")
        .assert()
        .success()
        .stdout(predicate::str::contains(job_id).and(predicate::str::contains("Completed")));

    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .args(["yank", job_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-key"))
        .stdout(predicate::str::contains("background job"));

    Ok(())
}
