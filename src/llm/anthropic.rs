use super::{LlmClient, LlmRequest, LlmResponse};
use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};

pub struct AnthropicClient {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl AnthropicClient {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

#[async_trait::async_trait]
impl LlmClient for AnthropicClient {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let body = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: request.max_tokens,
            system: request.system_prompt,
            messages: vec![Message {
                role: "user".to_string(),
                content: request.user_message,
            }],
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(eyre!(
                "Anthropic API error {} (model: {}): {}",
                status,
                self.model,
                error_text
            ));
        }

        let api_response: AnthropicResponse = response.json().await?;

        let content = api_response
            .content
            .first()
            .and_then(|block| block.text.clone())
            .unwrap_or_default();

        Ok(LlmResponse { content })
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }
}
