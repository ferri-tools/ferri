use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_ctx_add_ls_rm() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let base_path = dir.path();

    // 1. Initialize project
    let mut cmd = Command::cargo_bin("ferri")?;
    cmd.current_dir(base_path).arg("init").assert().success();

    // 2. Create a test file
    let file_path = base_path.join("test.txt");
    fs::write(&file_path, "hello")?;

    // 3. Add the file to the context
    Command::cargo_bin("ferri")?
        .current_dir(base_path)
        .args(["ctx", "add", "test.txt"])
        .assert()
        .success();

    // 4. List the context to verify
    Command::cargo_bin("ferri")?
        .current_dir(base_path)
        .args(["ctx", "ls"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test.txt"));

    // 5. Remove the file from the context
    Command::cargo_bin("ferri")?
        .current_dir(base_path)
        .args(["ctx", "rm", "test.txt"])
        .assert()
        .success();

    // 6. List the context again to verify removal
    Command::cargo_bin("ferri")?
        .current_dir(base_path)
        .args(["ctx", "ls"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Context is empty."));

    Ok(())
}
