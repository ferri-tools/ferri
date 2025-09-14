use std::fs;
use std::path::Path;

/// Initializes a Ferri project by creating a `.ferri` directory in the specified base path.
///
/// # Arguments
///
/// * `base_path` - The path where the `.ferri` directory should be created.
///
/// # Errors
///
/// This function will return an error if the directory cannot be created for reasons
/// other than it already existing (e.g., permissions).
pub fn initialize_project(base_path: &Path) -> std::io::Result<()> {
    let ferri_dir = base_path.join(".ferri");
    fs::create_dir_all(ferri_dir)?;
    Ok(())
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_initialize_project_creates_directory() {
        // Create a temporary directory for the test.
        let dir = tempdir().unwrap();
        let base_path = dir.path();

        // The .ferri directory should not exist yet.
        let ferri_dir = base_path.join(".ferri");
        assert!(!ferri_dir.exists());

        // Call the function to create the directory.
        let result = initialize_project(base_path);
        assert!(result.is_ok());

        // The .ferri directory should now exist.
        assert!(ferri_dir.exists());
        assert!(ferri_dir.is_dir());

        // Calling it again should also succeed.
        let result_again = initialize_project(base_path);
        assert!(result_again.is_ok());
    }
}
