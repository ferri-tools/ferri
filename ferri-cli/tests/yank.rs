use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
fn test_ferri_yank() {
    let mut cmd = Command::cargo_bin("ferri").unwrap();
    cmd.arg("init").assert().success();

    let mut run_cmd = Command::cargo_bin("ferri").unwrap();
    run_cmd
        .arg("run")
        .arg("--")
        .arg("echo")
        .arg("yank test");

    let output = run_cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let job_id_line = stdout.lines().find(|line| line.contains("Successfully submitted job")).unwrap();
    let job_id = job_id_line.split("'").collect::<Vec<&str>>()[1];

    // Wait for the job to finish
    thread::sleep(Duration::from_millis(100));

    let mut yank_cmd = Command::cargo_bin("ferri").unwrap();
    yank_cmd.arg("yank").arg(job_id);

    yank_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("yank test"));
}
