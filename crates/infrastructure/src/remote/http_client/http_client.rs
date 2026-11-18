//! Безопасный HTTP-клиент для скачивания справочных данных.
//!
//! Ограничения (защита от подмены и злоупотреблений):
//! - только HTTPS;
//! - только хосты из [`ALLOWED_HOSTS`] (GitHub);
//! - без перенаправлений: иначе сервер мог бы увести запрос на чужой хост;
//! - ограничение размера ответа и времени ожидания.

use std::time::Duration;

use thiserror::Error;
use ureq::Agent;

/// Хосты, с которых разрешено скачивать данные.
pub const ALLOWED_HOSTS: &[&str] = &["raw.githubusercontent.com", "api.github.com"];

/// Максимальный размер текстового ответа (самый большой файл Paimon.moe ≈ 0,5 МБ).
const MAX_TEXT_BYTES: u64 = 8 * 1024 * 1024;
/// Максимальный размер картинки (картинки Paimon.moe — десятки килобайт).
const MAX_IMAGE_BYTES: u64 = 2 * 1024 * 1024;
/// Максимальное время на один запрос целиком.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
/// Схема адреса, которая допускается.
const HTTPS_PREFIX: &str = "https://";
/// Как приложение представляется серверу (GitHub API требует User-Agent).
const USER_AGENT: &str = concat!("WishCounter/", env!("CARGO_PKG_VERSION"));

/// Ошибка HTTP-запроса.
#[derive(Debug, Error)]
pub enum HttpError {
    /// Адрес не HTTPS или хост не входит в список разрешённых.
    #[error("адрес не разрешён: {0}")]
    UrlNotAllowed(String),
    /// Сервер ответил кодом ошибки (404, 500, ...).
    #[error("сервер вернул код {status} для {url}")]
    Status {
        /// HTTP-код ответа.
        status: u16,
        /// Адрес запроса.
        url: String,
    },
    /// Сеть недоступна, тайм-аут, слишком большой ответ и т.п.
    #[error("ошибка сети при запросе {url}: {reason}")]
    Network {
        /// Адрес запроса.
        url: String,
        /// Описание причины.
        reason: String,
    },
}

/// HTTP-клиент с ограничениями безопасности.
pub struct HttpClient {
    agent: Agent,
}

impl HttpClient {
    /// Создаёт клиент с тайм-аутом, без перенаправлений, только HTTPS.
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

    /// Скачивает текст по адресу (не больше `MAX_TEXT_BYTES` = 8 МБ).
    pub fn get_text(&self, url: &str) -> Result<String, HttpError> {
        let mut response = self.get(url)?;
        response
            .body_mut()
            .with_config()
            .limit(MAX_TEXT_BYTES)
            .read_to_string()
            .map_err(|error| to_http_error(url, error))
    }

    /// Скачивает двоичные данные (картинку) по адресу (не больше `MAX_IMAGE_BYTES` = 2 МБ).
    pub fn get_bytes(&self, url: &str) -> Result<Vec<u8>, HttpError> {
        let mut response = self.get(url)?;
        response
            .body_mut()
            .with_config()
            .limit(MAX_IMAGE_BYTES)
            .read_to_vec()
            .map_err(|error| to_http_error(url, error))
    }

    /// Проверяет адрес и выполняет GET-запрос; тело ответа читает вызывающий.
    fn get(&self, url: &str) -> Result<ureq::http::Response<ureq::Body>, HttpError> {
        check_url(url)?;
        self.agent
            .get(url)
            .call()
            .map_err(|error| to_http_error(url, error))
    }
}

impl Default for HttpClient {
    /// То же, что [`HttpClient::new`].
    fn default() -> Self {
        Self::new()
    }
}

/// Проверяет, что адрес — HTTPS на разрешённом хосте.
///
/// Хост — часть между `https://` и первым `/`. Символы `@` и `:` в ней
/// запрещены: так нельзя подставить другой хост через `user@host` или порт.
fn check_url(url: &str) -> Result<(), HttpError> {
    let rest = url
        .strip_prefix(HTTPS_PREFIX)
        .ok_or_else(|| HttpError::UrlNotAllowed(url.to_owned()))?;
    let host = rest.split('/').next().unwrap_or_default();
    let allowed = !host.contains(['@', ':']) && ALLOWED_HOSTS.contains(&host);
    if allowed {
        Ok(())
    } else {
        Err(HttpError::UrlNotAllowed(url.to_owned()))
    }
}

/// Превращает ошибку ureq в [`HttpError`].
fn to_http_error(url: &str, error: ureq::Error) -> HttpError {
    match error {
        ureq::Error::StatusCode(status) => HttpError::Status {
            status,
            url: url.to_owned(),
        },
        other => HttpError::Network {
            url: url.to_owned(),
            reason: other.to_string(),
        },
    }
}

// Тесты лежат рядом, в файле `http_client.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "http_client.tests.rs"]
mod tests;
