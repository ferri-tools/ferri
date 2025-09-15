use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn test_init_command() {
    // Create a temporary directory for the test.
    let dir = tempdir().unwrap();
    let base_path = dir.path();

    // Run the `init` command.
    let mut cmd = Command::cargo_bin("ferri-cli").unwrap();
    cmd.current_dir(base_path).arg("init").assert().success();

    // Check that the .ferri directory and files were created.
    let ferri_dir = base_path.join(".ferri");
    assert!(ferri_dir.exists());
    assert!(ferri_dir.is_dir());

    let secrets_path = ferri_dir.join("secrets.json");
    assert!(secrets_path.exists());
    assert!(secrets_path.is_file());
    assert_eq!(std::fs::read_to_string(secrets_path).unwrap(), "{}");

    let models_path = ferri_dir.join("models.json");
    assert!(models_path.exists());
    assert!(models_path.is_file());
    assert_eq!(std::fs::read_to_string(models_path).unwrap(), "[]");

    let context_path = ferri_dir.join("context.json");
    assert!(context_path.exists());
    assert!(context_path.is_file());
    assert_eq!(std::fs::read_to_string(context_path).unwrap(), "[]");
}
