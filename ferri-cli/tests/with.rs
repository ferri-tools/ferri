use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn test_with_simple_command() {
    let dir = tempdir().unwrap();
    let base_path = dir.path();

    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("with")
        .arg("--")
        .arg("echo")
        .arg("it works")
        .assert()
        .success()
        .stdout("it works\n");
}

#[test]
fn test_with_secret_injection() {
    let dir = tempdir().unwrap();
    let base_path = dir.path();

    // 1. Initialize
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("init")
        .assert()
        .success();

    // 2. Set a secret
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("secrets")
        .arg("set")
        .arg("MY_TEST_SECRET")
        .arg("hello_from_secret")
        .assert()
        .success();

    // 3. Run a command that prints the environment variable
    // Note: `printenv` is not available on Windows. A more robust test
    // would use a custom script.
    if cfg!(windows) {
        println!("Skipping `printenv` test on Windows.");
        return;
    }
    
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("with")
        .arg("--")
        .arg("printenv")
        .arg("MY_TEST_SECRET")
        .assert()
        .success()
        .stdout("hello_from_secret\n");
}

