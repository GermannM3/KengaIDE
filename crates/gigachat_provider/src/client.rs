//! HTTP-клиент для GigaChat API.

use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::auth::AuthManager;
use crate::error::GigaChatError;
use crate::models::GigaChatModel;

const CHAT_URL: &str = "https://gigachat.devices.sberbank.ru/api/v1/chat/completions";

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Option<Vec<ChatChoice>>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: Option<ChatChoiceMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    total_tokens: Option<u32>,
}

pub struct GigaChatClient {
    auth: AuthManager,
    http_client: reqwest::Client,
    model: GigaChatModel,
}

impl GigaChatClient {
    pub fn new(auth: AuthManager, http_client: reqwest::Client) -> Self {
        Self {
            auth,
            http_client,
            model: GigaChatModel::GigaChatUltra,
        }
    }

    pub async fn chat(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<(String, Option<u32>), GigaChatError> {
        let token = self.auth.get_token().await?;

        let request = ChatRequest {
            model: self.model.as_str().to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
        };

        let start = Instant::now();

        let response = self
            .send_with_retry(&token, &request)
            .await?;

        let _latency_ms = start.elapsed().as_millis() as u64;

        let content = response
            .choices
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.message)
            .and_then(|m| m.content)
            .unwrap_or_else(String::new);

        let tokens_used = response.usage.and_then(|u| u.total_tokens);

        Ok((content, tokens_used))
    }

    async fn send_with_retry(
        &self,
        token: &str,
        request: &ChatRequest,
    ) -> Result<ChatResponse, GigaChatError> {
        const MAX_RETRIES: u32 = 3;

        for attempt in 0..MAX_RETRIES {
            let result = self.send_once(token, request).await;

            match result {
                Ok(r) => return Ok(r),
                Err(e) => {
                    let is_retryable = match &e {
                        GigaChatError::Http(_) => true,
                        GigaChatError::Api(msg) => {
                            msg.contains("500") || msg.contains("502") || msg.contains("503")
                        }
                        _ => false,
                    };
                    if is_retryable && attempt < MAX_RETRIES - 1 {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            500 * (attempt + 1) as u64,
                        ))
                        .await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }

        Err(GigaChatError::Api("Max retries exceeded".to_string()))
    }

    pub async fn healthcheck(&self) -> Result<bool, GigaChatError> {
        let token = self.auth.get_token().await?;
        Ok(!token.is_empty())
    }

    async fn send_once(
        &self,
        token: &str,
        request: &ChatRequest,
    ) -> Result<ChatResponse, GigaChatError> {
        let response = self
            .http_client
            .post(CHAT_URL)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| GigaChatError::Http(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| GigaChatError::Http(e.to_string()))?;

        if !status.is_success() {
            return Err(GigaChatError::Api(format!(
                "status {}: {}",
                status, body
            )));
        }

        serde_json::from_str(&body).map_err(|e| GigaChatError::Api(e.to_string()))
    }
}
