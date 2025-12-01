use crate::message::{ContentBlock, Message};
use crate::models::Model;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    pub content: Vec<ContentBlock>,
    pub usage_metrics: Option<TokenUsage>,
    pub raw_response: serde_json::Value,
}

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn prepare_assets(&self, messages: &[Message]) -> Result<Vec<Message>>;

    async fn generate(
        &self,
        model_config: &Model,
        messages: &[Message],
        stream: bool,
    ) -> Result<GenerationResponse>;
}
