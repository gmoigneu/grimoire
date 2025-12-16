mod anthropic;
mod openai;

pub use anthropic::AnthropicClient;
pub use openai::OpenAIClient;

use color_eyre::eyre::Result;

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub system_prompt: String,
    pub user_message: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
}

#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse>;
    #[allow(dead_code)]
    fn is_configured(&self) -> bool;
}

/// Helper to select the best available client
pub fn get_client(
    anthropic_key: Option<&str>,
    anthropic_model: Option<&str>,
    openai_key: Option<&str>,
) -> Option<Box<dyn LlmClient>> {
    // Prefer Anthropic
    if let Some(key) = anthropic_key {
        if !key.is_empty() {
            let model = anthropic_model.unwrap_or("claude-sonnet-4-20250514");
            return Some(Box::new(AnthropicClient::new(key, model)));
        }
    }

    // Fall back to OpenAI
    if let Some(key) = openai_key {
        if !key.is_empty() {
            return Some(Box::new(OpenAIClient::new(key)));
        }
    }

    None
}

/// Synchronous LLM completion using blocking tokio runtime
pub fn complete_sync(
    anthropic_key: Option<&str>,
    anthropic_model: Option<&str>,
    openai_key: Option<&str>,
    request: LlmRequest,
) -> Result<LlmResponse> {
    let client = get_client(anthropic_key, anthropic_model, openai_key)
        .ok_or_else(|| color_eyre::eyre::eyre!("No LLM API key configured. Go to Settings (s) to add one."))?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(client.complete(request))
}
