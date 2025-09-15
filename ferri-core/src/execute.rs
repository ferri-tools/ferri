//! Core logic for executing commands with injected context.

use crate::secrets; // Import the secrets module
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

/// Executes a command with secrets injected as environment variables.
///
/// # Arguments
///
/// * `base_path` - The root of the project.
/// * `command_with_args` - A vector where the first element is the command
///   and the rest are its arguments.
///
/// # Errors
///
/// Returns an error if secrets cannot be read or if the command fails.
pub fn execute_with_context(
    base_path: &Path,
    command_with_args: &[String],
) -> io::Result<()> {
    if command_with_args.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No command provided to execute.",
        ));
    }

    // 1. Load and decrypt secrets
    let decrypted_secrets = secrets::read_all_secrets(base_path)?;

    // 2. Prepare the command
    let command = &command_with_args[0];
    let args = &command_with_args[1..];

    let mut child = Command::new(command)
        .args(args)
        .envs(decrypted_secrets) // Inject secrets as environment variables
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let status = child.wait()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Command failed with exit code {}", status),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::initialize_project;
    use tempfile::tempdir;

    #[test]
    fn test_simple_command_execution() {
        let dir = tempdir().unwrap();
        let base_path = dir.path();
        initialize_project(base_path).unwrap();
        
        let command = vec!["echo".to_string(), "hello".to_string()];
        let result = execute_with_context(base_path, &command);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_failing_command() {
        let dir = tempdir().unwrap();
        let base_path = dir.path();
        initialize_project(base_path).unwrap();
        
        let command = vec!["false".to_string()];
        let result = execute_with_context(base_path, &command);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_secret_injection() {
        let dir = tempdir().unwrap();
        let base_path = dir.path();
        initialize_project(base_path).unwrap();

        // Set a secret first
        secrets::set_secret(base_path, "MY_TEST_VAR", "hello_secret").unwrap();

        // Prepare a command that prints an environment variable.
        // This is OS-specific. `printenv` is common on Unix.
        // For a cross-platform test, a script would be better, but this is fine for now.
        let command = vec!["printenv".to_string(), "MY_TEST_VAR".to_string()];
        
        // We can't easily capture stdout here, so we're primarily testing
        // that the `execute_with_context` function can be called successfully
        // after secrets have been set. The integration test will verify the output.
        let result = execute_with_context(base_path, &command);
        assert!(result.is_ok());
    }
}
