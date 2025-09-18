//! Core logic for managing the project's context.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ContentType {
    Text,
    Png,
    Jpeg,
    WebP,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct ContextFile {
    pub path: String,
    pub content_type: ContentType,
}

/// Adds a set of file or directory paths to the context.
pub fn add_to_context(base_path: &Path, paths: Vec<PathBuf>) -> io::Result<()> {
    let context_path = base_path.join(".ferri").join("context.json");
    let mut current_files = read_context_file(&context_path)?;

    for path_buf in paths {
        let absolute_path = fs::canonicalize(&path_buf)?;
        let path_str = absolute_path.to_string_lossy().to_string();

        let content_type = match absolute_path.extension().and_then(|s| s.to_str()) {
            Some("png") => ContentType::Png,
            Some("jpg") | Some("jpeg") => ContentType::Jpeg,
            Some("webp") => ContentType::WebP,
            _ => ContentType::Text, // Default to text
        };

        let context_file = ContextFile {
            path: path_str,
            content_type,
        };

        current_files.insert(context_file);
    }

    write_context_file(&context_path, &current_files)?;
    Ok(())
}

/// Lists the paths currently in the context.
pub fn list_context(base_path: &Path) -> io::Result<Vec<String>> {
    let context_path = base_path.join(".ferri").join("context.json");
    let files = read_context_file(&context_path)?;
    let paths: Vec<String> = files.into_iter().map(|f| f.path).collect();
    Ok(paths)
}

/// Removes a set of file or directory paths from the context.
pub fn remove_from_context(base_path: &Path, paths: Vec<PathBuf>) -> io::Result<()> {
    let context_path = base_path.join(".ferri").join("context.json");
    let mut current_files = read_context_file(&context_path)?;
    let mut removals = HashSet::new();

    for path_buf in paths {
        let absolute_path = fs::canonicalize(&path_buf)?;
        removals.insert(absolute_path.to_string_lossy().to_string());
    }

    current_files.retain(|f| !removals.contains(&f.path));

    write_context_file(&context_path, &current_files)?;
    Ok(())
}

// --- Internal Helper Functions ---

fn read_context_file(path: &Path) -> io::Result<HashSet<ContextFile>> {
    if !path.exists() {
        return Ok(HashSet::new());
    }
    let content = fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(HashSet::new());
    }
    serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn write_context_file(path: &Path, files: &HashSet<ContextFile>) -> io::Result<()> {
    let content = serde_json::to_string_pretty(files)?;
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())
}

/// Reads all text files from the context and concatenates their content.
/// NOTE: This function will need to be updated to handle multimodal context.
pub fn get_full_context(base_path: &Path) -> io::Result<String> {
    let context_path = base_path.join(".ferri").join("context.json");
    let files = read_context_file(&context_path)?;
    let mut full_context = String::new();

    for context_file in files {
        if matches!(context_file.content_type, ContentType::Text) {
            let content = fs::read_to_string(&context_file.path)?;
            full_context.push_str(&format!(
                "\n--- File: {} ---\n{}\n",
                context_file.path, content
            ));
        }
        // Image files are ignored by this function for now.
    }

    Ok(full_context)
}

#[derive(Debug, Default)]
pub struct FullContext {
    pub text_content: String,
    pub image_files: Vec<ContextFile>,
}

/// Reads all files from the context, separating text and images.
pub fn get_full_multimodal_context(base_path: &Path) -> io::Result<FullContext> {
    let context_path = base_path.join(".ferri").join("context.json");
    let files = read_context_file(&context_path)?;
    let mut full_context = FullContext::default();

    for context_file in files {
        match context_file.content_type {
            ContentType::Text => {
                let content = fs::read_to_string(&context_file.path)?;
                full_context.text_content.push_str(&format!(
                    "\n--- File: {} ---\n{}\n",
                    context_file.path, content
                ));
            }
            ContentType::Png | ContentType::Jpeg | ContentType::WebP => {
                full_context.image_files.push(context_file);
            }
        }
    }

    Ok(full_context)
}
