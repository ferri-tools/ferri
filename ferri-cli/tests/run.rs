use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_ferri_run() {
    let mut cmd = Command::cargo_bin("ferri").unwrap();
    cmd.arg("init").assert().success();

    let mut run_cmd = Command::cargo_bin("ferri").unwrap();
    run_cmd
        .arg("run")
        .arg("--")
        .arg("echo")
        .arg("hello from test");

    run_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully submitted job"));
}
