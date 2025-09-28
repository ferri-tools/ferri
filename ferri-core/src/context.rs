//! Core logic for managing the project's context.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
struct Context {
    files: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ContentType {
    Text,
    Png,
    Jpeg,
    WebP,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct MultimodalFile {
    pub path: PathBuf,
    pub content_type: ContentType,
}

#[derive(Debug, Clone)]
pub struct MultimodalContext {
    pub text_content: String,
    pub image_files: Vec<MultimodalFile>,
}

/// Adds one or more files/directories to the context.
pub fn add_to_context(base_path: &Path, paths: Vec<PathBuf>) -> io::Result<()> {
    let context_path = base_path.join(".ferri").join("context.json");
    let mut context = read_context_file(&context_path)?;

    for path in paths {
        let canonical_path = path.canonicalize()?;
        let path_str = canonical_path.to_string_lossy().to_string();
        if !context.files.contains(&path_str) {
            context.files.push(path_str);
        }
    }

    write_context_file(&context_path, &context)
}

/// Lists the current context.
pub fn list_context(base_path: &Path) -> io::Result<Vec<String>> {
    let context_path = base_path.join(".ferri").join("context.json");
    let context = read_context_file(&context_path)?;
    Ok(context.files)
}

/// Removes one or more files/directories from the context.
pub fn remove_from_context(base_path: &Path, paths: Vec<PathBuf>) -> io::Result<()> {
    let context_path = base_path.join(".ferri").join("context.json");
    let mut context = read_context_file(&context_path)?;

    for path in paths {
        let canonical_path = path.canonicalize()?;
        let path_str = canonical_path.to_string_lossy().to_string();
        context.files.retain(|f| f != &path_str);
    }

    write_context_file(&context_path, &context)
}

/// Clears the entire context.
pub fn clear_context(base_path: &Path) -> io::Result<()> {
    let context_path = base_path.join(".ferri").join("context.json");
    let new_context = Context { files: Vec::new() };
    write_context_file(&context_path, &new_context)
}

/// Reads all files in the context and concatenates their content.
pub fn get_full_context(base_path: &Path) -> io::Result<String> {
    let multimodal_context = get_full_multimodal_context(base_path)?;
    Ok(multimodal_context.text_content)
}

/// Reads all files, separating text and images for multimodal models.
pub fn get_full_multimodal_context(base_path: &Path) -> io::Result<MultimodalContext> {
    let files = list_context(base_path)?;
    let mut text_content = String::new();
    let mut image_files = Vec::new();

    for file_path_str in files {
        let path = PathBuf::from(file_path_str);
        if path.is_dir() {
            // Recursively walk directories
            for entry in walkdir::WalkDir::new(path) {
                let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                if entry.path().is_file() {
                    process_file(entry.path(), &mut text_content, &mut image_files, base_path)?;
                }
            }
        } else if path.is_file() {
            process_file(&path, &mut text_content, &mut image_files, base_path)?;
        }
    }

    Ok(MultimodalContext { text_content, image_files })
}

fn process_file(path: &Path, text_content: &mut String, image_files: &mut Vec<MultimodalFile>, base_path: &Path) -> io::Result<()> {
    let content_type = get_content_type(path);
    let display_path = path.strip_prefix(base_path).unwrap_or(path);

    match content_type {
        ContentType::Text => {
            let content = fs::read_to_string(path)?;
            text_content.push_str(&format!("--- Content of file: {} ---\n", display_path.display()));
            text_content.push_str(&content);
            text_content.push_str("\n\n");
        }
        ContentType::Png | ContentType::Jpeg | ContentType::WebP => {
            image_files.push(MultimodalFile {
                path: path.to_path_buf(),
                content_type,
            });
        }
        ContentType::Unknown => {
            // For now, we try to read it as text and ignore errors.
            if let Ok(content) = fs::read_to_string(path) {
                text_content.push_str(&format!("--- Content of file: {} ---\n", display_path.display()));
                text_content.push_str(&content);
                text_content.push_str("\n\n");
            }
        }
    }
    Ok(())
}

fn get_content_type(path: &Path) -> ContentType {
    match path.extension().and_then(|s| s.to_str()) {
        Some("png") => ContentType::Png,
        Some("jpg") | Some("jpeg") => ContentType::Jpeg,
        Some("webp") => ContentType::WebP,
        Some("txt") | Some("md") | Some("rs") | Some("py") | Some("js") | Some("ts") | Some("html") | Some("css") | Some("json") | Some("yaml") | Some("toml") => ContentType::Text,
        _ => ContentType::Unknown,
    }
}

// --- Internal Helper Functions ---

fn read_context_file(path: &Path) -> io::Result<Context> {
    if !path.exists() {
        return Ok(Context { files: Vec::new() });
    }
    let content = fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(Context { files: Vec::new() });
    }
    serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn write_context_file(path: &Path, context: &Context) -> io::Result<()> {
    let content = serde_json::to_string_pretty(context)?;
    fs::write(path, content)
}