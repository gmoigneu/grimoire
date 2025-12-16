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

/// Helper to create a client based on provider
pub fn get_client(provider: &str, api_key: &str, model: &str) -> Option<Box<dyn LlmClient>> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return None;
    }

    match provider.to_lowercase().as_str() {
        "openai" => Some(Box::new(OpenAIClient::new(api_key))),
        _ => {
            // Default to Anthropic
            let model = if model.is_empty() {
                "claude-sonnet-4-20250514"
            } else {
                model
            };
            Some(Box::new(AnthropicClient::new(api_key, model)))
        }
    }
}

/// Synchronous LLM completion using blocking tokio runtime
pub fn complete_sync(
    provider: &str,
    api_key: &str,
    model: &str,
    request: LlmRequest,
) -> Result<LlmResponse> {
    let client = get_client(provider, api_key, model).ok_or_else(|| {
        color_eyre::eyre::eyre!("No LLM API key configured. Go to Settings (s) to add one.")
    })?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(client.complete(request))
}
