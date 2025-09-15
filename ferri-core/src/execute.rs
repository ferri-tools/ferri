//! Core logic for executing commands with injected context.

use crate::{context, secrets}; // Import the context and secrets modules
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::fs;

/// Executes a command with secrets and context injected.
///
/// # Arguments
///
/// * `base_path` - The root of the project.
/// * `command_with_args` - A vector where the first element is the command
///   and the rest are its arguments.
///
/// # Errors
///
/// Returns an error if secrets/context cannot be read or if the command fails.
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

    // 1. Load secrets and context
    let decrypted_secrets = secrets::read_all_secrets(base_path)?;
    let context_files = context::list_context(base_path)?;

    // 2. Prepare the command and arguments
    let command = &command_with_args[0];
    let mut args = command_with_args[1..].to_vec();

    // 3. Inject context
    if !context_files.is_empty() {
        let mut full_context = String::new();
        for file_path_str in context_files {
            let file_path = base_path.join(file_path_str);
            if file_path.exists() {
                let content = fs::read_to_string(file_path)?;
                full_context.push_str(&content);
                full_context.push('\n'); // Add a newline between file contents
            }
        }

        // Prepend the combined context to the *last* argument (usually the prompt)
        if let Some(last_arg) = args.last_mut() {
            *last_arg = format!("{}{}", full_context, last_arg);
        }
    }

    // 4. Execute the command with secrets
    let mut child = Command::new(command)
        .args(&args)
        .envs(decrypted_secrets)
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
    use crate::{context, initialize_project, secrets};
    use std::path::PathBuf; // Add missing import
    use tempfile::tempdir;
    use std::fs;

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
    fn test_secret_injection() {
        let dir = tempdir().unwrap();
        let base_path = dir.path();
        initialize_project(base_path).unwrap();
        secrets::set_secret(base_path, "MY_TEST_VAR", "hello_secret").unwrap();

        let command = vec!["printenv".to_string(), "MY_TEST_VAR".to_string()];
        let result = execute_with_context(base_path, &command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_context_injection() {
        let dir = tempdir().unwrap();
        let base_path = dir.path();
        initialize_project(base_path).unwrap();

        // Create a dummy file and add it to the context
        let dummy_file = base_path.join("dummy.txt");
        fs::write(&dummy_file, "dummy content").unwrap();
        // We must add the path as a string, not a PathBuf, to match the CLI behavior
        context::add_to_context(base_path, vec![PathBuf::from("dummy.txt")]).unwrap();

        let command = vec!["echo".to_string(), "prompt".to_string()];
        let result = execute_with_context(base_path, &command);
        assert!(result.is_ok());
    }
}
