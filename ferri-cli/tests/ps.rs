use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_ferri_ps() {
    let mut cmd = Command::cargo_bin("ferri").unwrap();
    cmd.arg("init").assert().success();

    let mut run_cmd = Command::cargo_bin("ferri").unwrap();
    run_cmd
        .arg("run")
        .arg("--")
        .arg("sleep")
        .arg("0.1");

    let output = run_cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let job_id_line = stdout.lines().find(|line| line.contains("Successfully submitted job")).unwrap();
    let job_id = job_id_line.split("'").collect::<Vec<&str>>()[1];

    let mut ps_cmd = Command::cargo_bin("ferri").unwrap();
    ps_cmd.arg("ps");

    ps_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(job_id));
}
