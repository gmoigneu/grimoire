use super::{LlmClient, LlmRequest, LlmResponse};
use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};

pub struct OpenAIClient {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAIClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: "gpt-4o".to_string(),
            client: reqwest::Client::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_model(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: Option<String>,
}

#[async_trait::async_trait]
impl LlmClient for OpenAIClient {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let mut messages = vec![Message {
            role: "system".to_string(),
            content: request.system_prompt,
        }];

        messages.push(Message {
            role: "user".to_string(),
            content: request.user_message,
        });

        let body = OpenAIRequest {
            model: self.model.clone(),
            max_tokens: request.max_tokens,
            messages,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(eyre!("OpenAI API error {}: {}", status, error_text));
        }

        let api_response: OpenAIResponse = response.json().await?;

        let content = api_response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .unwrap_or_default();

        Ok(LlmResponse { content })
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }
}
