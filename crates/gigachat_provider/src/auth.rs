//! OAuth2 для GigaChat API.

use base64::Engine;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::error::GigaChatError;

const OAUTH_URL: &str = "https://ngw.devices.sberbank.ru:9443/api/v2/oauth";
const SCOPE_PERS: &str = "GIGACHAT_API_PERS";

#[derive(Debug, Deserialize)]
struct OAuthResponse {
    access_token: String,
    expires_at: i64,
}

#[derive(Clone)]
struct TokenState {
    token: String,
    expires_at: i64,
}

/// OAuth-менеджер. Кеширует токен, обновляет до истечения.
pub struct AuthManager {
    client_id: String,
    client_secret: String,
    state: Arc<RwLock<Option<TokenState>>>,
    http_client: reqwest::Client,
}

impl AuthManager {
    pub fn new(
        client_id: String,
        client_secret: String,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            state: Arc::new(RwLock::new(None)),
            http_client,
        }
    }

    /// Возвращает валидный access token. Обновляет при необходимости.
    pub async fn get_token(&self) -> Result<String, GigaChatError> {
        {
            let state = self.state.read().await;
            if let Some(ref s) = *state {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| GigaChatError::OAuth(e.to_string()))?
                    .as_secs() as i64;
                if s.expires_at > now + 60 {
                    return Ok(s.token.clone());
                }
            }
        }

        self.refresh_token().await
    }

    async fn refresh_token(&self) -> Result<String, GigaChatError> {
        let auth_key = base64::engine::general_purpose::STANDARD.encode(format!(
            "{}:{}",
            self.client_id, self.client_secret
        ));

        let rquid = uuid::Uuid::new_v4().to_string();

        let response = self
            .http_client
            .post(OAUTH_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .header("RqUID", &rquid)
            .header("Authorization", format!("Basic {}", auth_key))
            .body(format!("scope={}", SCOPE_PERS))
            .send()
            .await
            .map_err(|e| GigaChatError::Http(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| GigaChatError::Http(e.to_string()))?;

        if !status.is_success() {
            return Err(GigaChatError::OAuth(format!(
                "status {}: {}",
                status, body
            )));
        }

        let oauth: OAuthResponse = serde_json::from_str(&body)
            .map_err(|e| GigaChatError::OAuth(e.to_string()))?;

        debug!("GigaChat OAuth: token refreshed, expires_at={}", oauth.expires_at);

        let state = TokenState {
            token: oauth.access_token.clone(),
            expires_at: oauth.expires_at,
        };

        {
            let mut s = self.state.write().await;
            *s = Some(state);
        }

        Ok(oauth.access_token)
    }
}
