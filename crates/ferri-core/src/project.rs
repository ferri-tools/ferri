use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;

/// Initializes a Ferri project by creating a `.ferri` directory and the default state files.
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
    
    let jobs_dir = ferri_dir.join("jobs");
    if !jobs_dir.exists() {
        fs::create_dir_all(jobs_dir)?;
    }

    Ok(())
}

/// Verifies that a `.ferri` directory exists in the given base path.
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

