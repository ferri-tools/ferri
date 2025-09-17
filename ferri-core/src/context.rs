//! Core logic for managing the project's context.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
struct Context {
    files: Vec<String>,
}

fn read_context(base_path: &Path) -> io::Result<Context> {
    let context_path = base_path.join(".ferri").join("context.json");
    let file_content = fs::read_to_string(context_path)?;

    // If the file is empty or invalid, treat it as a new context
    if file_content.trim().is_empty() {
        return Ok(Context { files: vec![] });
    }

    serde_json::from_str(&file_content)
        .map_err(|e| {
            // Provide a more helpful error message
            let error_msg = format!(
                "Failed to parse '.ferri/context.json'. It might be corrupted. Error: {}",
                e
            );
            io::Error::new(io::ErrorKind::InvalidData, error_msg)
        })
}

fn write_context(base_path: &Path, context: &Context) -> io::Result<()> {
    let context_path = base_path.join(".ferri").join("context.json");
    let file_content = serde_json::to_string_pretty(context)?;
    fs::write(context_path, file_content)
}

pub fn add_to_context(base_path: &Path, paths: Vec<PathBuf>) -> io::Result<()> {
    let mut context = read_context(base_path)?;
    for path in paths {
        let path_str = path.to_string_lossy().to_string();
        if !context.files.contains(&path_str) {
            context.files.push(path_str);
        }
    }
    write_context(base_path, &context)
}

pub fn list_context(base_path: &Path) -> io::Result<Vec<String>> {
    let context = read_context(base_path)?;
    Ok(context.files)
}

pub fn remove_from_context(base_path: &Path, paths: Vec<PathBuf>) -> io::Result<()> {
    let mut context = read_context(base_path)?;
    for path in paths {
        let path_str = path.to_string_lossy().to_string();
        context.files.retain(|f| f != &path_str);
    }
    write_context(base_path, &context)
}
