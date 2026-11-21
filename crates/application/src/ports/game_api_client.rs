//! Порт: HTTP-клиент для запросов к API игры.
//!
//! В отличие от [`crate::reference_downloader::ReferenceDownloader`], этот клиент
//! обращается к серверам HoYoverse, а не к GitHub. Поэтому хосты не ограничены
//! белым списком — адрес для каждого запроса передаётся явно, а источник URL
//! (веб-кэш игры) гарантирует, что запрос идёт туда, куда нужно.

/// Клиент API игры: делает GET-запрос и возвращает тело как строку.
///
/// Реализация — в `infrastructure::remote::HttpClientGameApi` (через `ureq`).
/// Порт нужен, чтобы сервис импорта в `application` не зависел от `ureq`.
pub trait GameApiClient: Send + Sync {
    /// Выполняет GET-запрос и возвращает тело ответа как строку.
    /// `url` — полный адрес, включая параметры (в том числе `authkey`).
    fn get(&self, url: &str) -> Result<String, GameApiError>;
}

/// Ошибка сетевого запроса к API игры.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GameApiError {
    /// Сетевая ошибка (нет соединения, тайм-аут и т.п.).
    #[error("сетевая ошибка: {0}")]
    Network(String),
    /// HTTP-ошибка (не 2xx).
    #[error("HTTP {0}")]
    Http(String),
}
