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
    let mut cmd = Command::cargo_bin("ferri")?;
    cmd.current_dir(base_path).arg("init").assert().success();

    // 2. Set a secret and add a model
    // We'll simulate a script that would use this secret.
    cmd = Command::cargo_bin("ferri")?;
    cmd.current_dir(base_path)
        .args(["secrets", "set", "MY_API_KEY", "test-key"])
        .assert()
        .success();

    // 3. Add a file to context
    let context_file = base_path.join("context.txt");
    fs::write(context_file, "Test context data.")?;
    cmd = Command::cargo_bin("ferri")?;
    cmd.current_dir(base_path)
        .args(["ctx", "add", "context.txt"])
        .assert()
        .success();

    // 4. Test `ferri with`
    // We'll use `printenv` to check for the secret and `echo` to check for context.
    // This is a simplified check. A real test would use a model.
    let mut with_cmd = Command::cargo_bin("ferri")?;
    with_cmd
        .current_dir(base_path)
        .args([
            "with",
            "--ctx",
            // Note: We are not using --model here as it requires a live model endpoint.
            // The core logic is tested by the secret and context injection.
            "--",
            "sh",
            "-c",
            "echo $MY_API_KEY && cat",
        ])
        .write_stdin("prompt")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-key"))
        .stdout(predicate::str::contains("Test context data."));


    // 5. Test `ferri run`, `ps`, and `yank`
    let mut run_cmd = Command::cargo_bin("ferri")?;
    let run_output = run_cmd
        .current_dir(base_path)
        .args([
            "run",
            "--ctx",
            "--",
            "sh",
            "-c",
            "echo $MY_API_KEY && echo 'background job'",
        ])
        .output()?;

    assert!(run_output.status.success());
    let stdout = String::from_utf8(run_output.stdout)?;
    let job_id = stdout
        .lines()
        .find(|line| line.contains("Successfully submitted job"))
        .and_then(|line| line.split(' ').nth(1))
        .ok_or("Could not parse job ID")?;

    // Give the job a moment to complete
    std::thread::sleep(std::time::Duration::from_millis(200));

    let mut ps_cmd = Command::cargo_bin("ferri")?;
    ps_cmd
        .current_dir(base_path)
        .arg("ps")
        .assert()
        .success()
        .stdout(predicate::str::contains(job_id).and(predicate::str::contains("Completed")));

    let mut yank_cmd = Command::cargo_bin("ferri")?;
    yank_cmd
        .current_dir(base_path)
        .args(["yank", job_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-key"))
        .stdout(predicate::str::contains("background job"));

    Ok(())
}
