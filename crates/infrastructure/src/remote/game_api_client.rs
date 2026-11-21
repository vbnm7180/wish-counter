//! HTTP-клиент для запросов к API истории молитв HoYoverse.
//!
//! В отличие от [`super::http_client::HttpClient`], этот клиент не ограничен
//! белым списком хостов: адрес берётся из веб-кэша самой игры и всегда ведёт
//! на серверы HoYoverse. Лимит ответа — 128 КБ (одна страница ≈ 20 записей).

use std::time::Duration;

use application::ports::{GameApiClient, GameApiClientError};
use ureq::Agent;

/// Максимальный размер ответа API (одна страница × 20 записей).
const MAX_RESPONSE_BYTES: u64 = 128 * 1024;

/// Тайм-аут одного запроса.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// User-Agent для запросов к HoYoverse API.
const USER_AGENT: &str = concat!("WishCounter/", env!("CARGO_PKG_VERSION"));

/// HTTP-клиент для API истории молитв игры.
pub struct GameApiHttpClient {
    agent: Agent,
}

impl GameApiHttpClient {
    /// Создаёт клиент: только HTTPS, без перенаправлений, тайм-аут 30 с.
    pub fn new() -> Self {
        let agent = Agent::config_builder()
            .timeout_global(Some(REQUEST_TIMEOUT))
            .https_only(true)
            .max_redirects(0)
            .user_agent(USER_AGENT)
            .build()
            .into();
        Self { agent }
    }
}

impl Default for GameApiHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GameApiClient for GameApiHttpClient {
    /// Выполняет GET-запрос к API и возвращает тело ответа как строку.
    fn get(&self, url: &str) -> Result<String, GameApiClientError> {
        let mut response = self
            .agent
            .get(url)
            .call()
            .map_err(|e| match e {
                ureq::Error::StatusCode(code) => {
                    GameApiClientError::Http(format!("HTTP {code}"))
                }
                other => GameApiClientError::Network(other.to_string()),
            })?;

        response
            .body_mut()
            .with_config()
            .limit(MAX_RESPONSE_BYTES)
            .read_to_string()
            .map_err(|e| GameApiClientError::Network(e.to_string()))
    }
}
