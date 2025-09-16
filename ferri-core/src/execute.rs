//! Core logic for executing commands with injected context.

use crate::{context, models, secrets}; // Import the context and secrets modules
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::fs;

/// Arguments for preparing a command for execution.
/// This mirrors the `SharedArgs` struct in `ferri-cli`.
pub struct ExecutionArgs {
    pub model: Option<String>,
    pub use_context: bool,
    pub command_with_args: Vec<String>,
}

/// Prepares a `std::process::Command` by injecting secrets and context.
///
/// This is the new, unified function for preparing a command to be run either
/// in the foreground (`with`) or background (`run`).
///
/// # Arguments
///
/// * `base_path` - The root of the project.
/// * `args` - The structured execution arguments.
///
/// # Errors
///
/// Returns an error if secrets, context, or models cannot be read.
pub fn prepare_command(
    base_path: &Path,
    args: &ExecutionArgs,
) -> io::Result<(Command, HashMap<String, String>)> {
    if args.command_with_args.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No command provided to execute.",
        ));
    }

    // 1. Load secrets
    let mut decrypted_secrets = secrets::read_all_secrets(base_path)?;

    // 2. Prepare the command and arguments
    let mut final_command_with_args = args.command_with_args.clone();

    // 3. Handle model logic
    if let Some(model_alias) = &args.model {
        let all_models = models::list_models(base_path)?;
        let model = all_models.iter().find(|m| m.alias == *model_alias).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("Model '{}' not found.", model_alias))
        })?;

        // If the model requires an API key, add it to the secrets map.
        if let Some(secret_name) = &model.api_key_secret {
            let api_key = secrets::read_secret(base_path, secret_name)?;
            decrypted_secrets.insert(secret_name.clone(), api_key);
        }

        // Prepend the appropriate model runner to the command
        // This is a simplified example. We'll need to make this more robust.
        match model.provider.as_str() {
            "ollama" => {
                final_command_with_args.insert(0, model.model_name.clone());
                final_command_with_args.insert(0, "run".to_string());
                final_command_with_args.insert(0, "ollama".to_string());
            }
            _ => {
                // For now, assume other providers are handled by scripts
                // that use the injected environment variables.
            }
        }
    }

    // 4. Inject context if requested
    if args.use_context {
        let context_files = context::list_context(base_path)?;
        if !context_files.is_empty() {
            let mut full_context = String::new();
            for file_path_str in context_files {
                let file_path = base_path.join(&file_path_str);
                if file_path.exists() {
                    let content = fs::read_to_string(file_path)?;
                    full_context.push_str(&format!(
                        "File: {}\nContent:\n{}\n\n",
                        file_path_str,
                        content
                    ));
                }
            }

            // Prepend the combined context to the *last* argument (usually the prompt)
            if let Some(last_arg) = final_command_with_args.last_mut() {
                *last_arg = format!(
                    "You are a helpful assistant. Use the following file content to answer the user's question.\n\n---\n{}\n---\n\nQuestion: {}",
                    full_context.trim(),
                    last_arg
                );
            }
        }
    }

    // 5. Create the Command object
    let command_name = &final_command_with_args[0];
    let command_args = &final_command_with_args[1..];
    let mut command = Command::new(command_name);
    command.args(command_args);

    Ok((command, decrypted_secrets))
}


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
