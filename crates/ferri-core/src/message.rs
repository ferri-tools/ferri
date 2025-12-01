use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents the physical location or state of a large asset.
/// This distinction is crucial for the "Reference vs. Value" strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetSource {
    /// A reference to a file on the local filesystem.
    /// This is the default state for user inputs.
    LocalPath(PathBuf),

    /// A reference to a remote resource, such as a Google File API URI
    /// or an S3 object URL. Used for "Director Mode" chaining.
    RemoteUri(String),

    /// Raw binary data held in memory.
    /// STRICTLY LIMITED to small assets (< 5MB) or generated thumbnails.
    /// Usage for video triggers a warning or error.
    Bytes(Vec<u8>),
}

/// Metadata describing the media asset.
/// Essential for Content-Type headers and optimizing provider selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub mime_type: String,
    pub size_bytes: Option<u64>,
    pub file_name: Option<String>,
}

/// The atomic unit of meaning in a Ferri conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentBlock {
    /// Pure UTF-8 text content.
    Text(String),

    /// An image asset.
    Image {
        source: AssetSource,
        metadata: MediaMetadata,
    },

    /// A video asset.
    /// Separated from Image to allow for video-specific fields (e.g., duration) later.
    Video {
        source: AssetSource,
        metadata: MediaMetadata,
    },

    /// An audio asset.
    Audio {
        source: AssetSource,
        metadata: MediaMetadata,
    },

    /// Result from a tool execution (future-proofing for L3 Agentic features).
    ToolResult {
        tool_use_id: String,
        output: String,
    },
}

/// Normalized roles across all providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// A unified message structure preserving temporal sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    /// The content is a vector, preserving the order of text/media interleaving.
    pub content: Vec<ContentBlock>,
}

/// The complete context of a conversation or prompt execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub messages: Vec<Message>,
}
