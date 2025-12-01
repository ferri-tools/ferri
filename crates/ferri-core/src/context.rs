//! Core logic for managing the project's context.

use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::message::{AssetSource, ContentBlock, MediaMetadata, Message, Role};

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
            for entry in WalkDir::new(path) {
                let entry = entry.map_err(io::Error::other)?;
                if entry.path().is_file() {
                    process_file(entry.path(), &mut text_content, &mut image_files, base_path)?;
                }
            }
        } else if path.is_file() {
            process_file(&path, &mut text_content, &mut image_files, base_path)?;
        }
    }

    Ok(MultimodalContext {
        text_content,
        image_files,
    })
}

fn process_file(
    path: &Path,
    text_content: &mut String,
    image_files: &mut Vec<MultimodalFile>,
    base_path: &Path,
) -> io::Result<()> {
    let content_type = get_content_type(path);
    let display_path = path.strip_prefix(base_path).unwrap_or(path);

    match content_type {
        ContentType::Text => {
            let content = fs::read_to_string(path)?;
            text_content.push_str(&format!(
                "--- Content of file: {} ---
",
                display_path.display()
            ));
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
                text_content.push_str(&format!(
                    "--- Content of file: {} ---
",
                    display_path.display()
                ));
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
        Some("txt") | Some("md") | Some("rs") | Some("py") | Some("js") | Some("ts")
        | Some("html") | Some("css") | Some("json") | Some("yaml") | Some("toml") => {
            ContentType::Text
        }
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

/// Implements the conversion from the legacy `MultimodalContext` to the new `Vec<Message>` format.
/// This is the "Shim Layer" that allows the new multimodal engine to process existing context.
impl TryFrom<MultimodalContext> for Vec<Message> {
    /// The error type returned if the conversion fails.
    type Error = anyhow::Error;

    /// Performs the conversion from `MultimodalContext` to `Vec<Message>`.
    fn try_from(legacy_ctx: MultimodalContext) -> Result<Self, Self::Error> {
        let mut content_blocks = Vec::new();

        // If there's any text content in the legacy context, add it as a single Text ContentBlock.
        // This preserves the original behavior of concatenating all text.
        if !legacy_ctx.text_content.is_empty() {
            content_blocks.push(ContentBlock::Text(legacy_ctx.text_content));
        }

        // Iterate over legacy image files and convert them into Image ContentBlocks.
        for img in legacy_ctx.image_files {
            // Determine the MIME type based on the legacy ContentType enum.
            let mime_type = match img.content_type {
                ContentType::Png => "image/png",
                ContentType::Jpeg => "image/jpeg",
                ContentType::WebP => "image/webp",
                // For any other ContentType, default to a generic octet-stream.
                // In the legacy system, only image types were explicitly handled as files.
                _ => "application/octet-stream",
            };

            // Create MediaMetadata for the image. Size is unknown in the legacy context.
            let metadata = MediaMetadata {
                mime_type: mime_type.to_string(),
                size_bytes: None, // Legacy context does not easily provide file size.
                file_name: img.path.file_name().map(|s| s.to_string_lossy().to_string()),
            };

            // Push a new Image ContentBlock, referencing the local path.
            content_blocks.push(ContentBlock::Image{
                source: AssetSource::LocalPath(img.path),
                metadata
            });
        }

        // Wrap all processed content blocks into a single Message with a User role.
        // The legacy context is always assumed to originate from user input.
        let message = Message {
            role: Role::User,
            content: content_blocks,
        };

        // Return the single message wrapped in a Vec, as `Vec<Message>` is the target type.
        Ok(vec![message])
    }
}
