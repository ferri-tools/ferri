pub mod context;
pub mod execute;
pub mod jobs;
pub mod models;
pub mod secrets;
pub mod agent;
pub mod flow;
pub mod logging;

use std::fs;
use std::path::Path;

/// Initializes a Ferri project by creating a `.ferri` directory and the default state files.
///
/// This function will create:
/// - A `.ferri` directory.
/// - An empty `.ferri/secrets.json`.
/// - An empty `.ferri/models.json`.
/// - An empty `.ferri/context.json`.
///
/// # Arguments
///
/// * `base_path` - The path where the `.ferri` directory should be created.
///
/// # Errors
///
/// This function will return an error if the directory or files cannot be created.
pub fn initialize_project(base_path: &Path) -> std::io::Result<()> {
    let ferri_dir = base_path.join(".ferri");
    fs::create_dir_all(&ferri_dir)?;

    let secrets_path = ferri_dir.join("secrets.json");
    if !secrets_path.exists() {
        fs::write(secrets_path, "{}")?;
    }

    let models_path = ferri_dir.join("models.json");
    if !models_path.exists() {
        fs::write(models_path, "[]")?;
    }

    let context_path = ferri_dir.join("context.json");
    if !context_path.exists() {
        fs::write(context_path, "{\n  \"files\": []\n}")?;
    }

    Ok(())
}


use std::io::{Error, ErrorKind};

/// Verifies that a `.ferri` directory exists in the given base path.
///
/// # Arguments
///
/// * `base_path` - The path to check for the `.ferri` directory.
///
/// # Errors
///
/// Returns an `io::Error` with `ErrorKind::NotFound` if the directory does not exist.
pub fn verify_project_initialized(base_path: &Path) -> std::io::Result<()> {
    let ferri_dir = base_path.join(".ferri");
    if !ferri_dir.exists() || !ferri_dir.is_dir() {
        return Err(Error::new(
            ErrorKind::NotFound,
            "Project not initialized. Please run `ferri init` first.",
        ));
    }
    Ok(())
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_initialize_project_creates_directory_and_files() {
        // Create a temporary directory for the test.
        let dir = tempdir().unwrap();
        let base_path = dir.path();

        // Paths for the directory and files.
        let ferri_dir = base_path.join(".ferri");
        let secrets_path = ferri_dir.join("secrets.json");
        let models_path = ferri_dir.join("models.json");
        let context_path = ferri_dir.join("context.json");

        // They should not exist yet.
        assert!(!ferri_dir.exists());

        // Call the function to create everything.
        let result = initialize_project(base_path);
        assert!(result.is_ok());

        // The .ferri directory and files should now exist.
        assert!(ferri_dir.exists() && ferri_dir.is_dir());
        assert!(secrets_path.exists() && secrets_path.is_file());
        assert!(models_path.exists() && models_path.is_file());
        assert!(context_path.exists() && context_path.is_file());

        // Check file contents
        assert_eq!(fs::read_to_string(&secrets_path).unwrap(), "{}");
        assert_eq!(fs::read_to_string(&models_path).unwrap(), "[]");
        assert_eq!(fs::read_to_string(&context_path).unwrap(), "{\n  \"files\": []\n}");

        // Calling it again should also succeed and not overwrite existing files.
        fs::write(&secrets_path, "{{\"key\":\"value\"}}").unwrap();
        let result_again = initialize_project(base_path);
        assert!(result_again.is_ok());
        assert_eq!(fs::read_to_string(&secrets_path).unwrap(), "{{\"key\":\"value\"}}");
    }

    #[test]
    fn test_add_and_remove_from_context() {
        let dir = tempdir().unwrap();
        let base_path = dir.path();
        initialize_project(base_path).unwrap();

        let path1 = PathBuf::from("file1.txt");
        let path2 = PathBuf::from("dir/file2.txt");
        fs::create_dir_all(base_path.join("dir")).unwrap();
        fs::write(base_path.join(&path1), "content1").unwrap();
        fs::write(base_path.join(&path2), "content2").unwrap();

        // Add paths
        context::add_to_context(base_path, vec![path1.clone(), path2.clone()]).unwrap();
        let mut current_context = context::list_context(base_path).unwrap();
        assert_eq!(current_context.len(), 2);
        assert!(current_context.contains(&"file1.txt".to_string()));

        // Remove one path
        context::remove_from_context(base_path, vec![path1]).unwrap();
        current_context = context::list_context(base_path).unwrap();
        assert_eq!(current_context.len(), 1);
        assert!(!current_context.contains(&"file1.txt".to_string()));
        assert!(current_context.contains(&"dir/file2.txt".to_string()));
    }
}