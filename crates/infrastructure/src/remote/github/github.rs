//! Чтение файлов из публичного репозитория GitHub.
//!
//! Файлы скачиваются по конкретной **ревизии** (идентификатору коммита),
//! а не по ветке: так все файлы одного обновления гарантированно относятся
//! к одному и тому же состоянию репозитория.

use serde::Deserialize;
use thiserror::Error;

use super::http_client::{HttpClient, HttpError};

/// Длина идентификатора коммита Git (SHA-1 в шестнадцатеричном виде).
const REVISION_LENGTH: usize = 40;
/// Ветка, последняя ревизия которой запрашивается.
const DEFAULT_BRANCH: &str = "main";

/// Ошибка работы с GitHub.
#[derive(Debug, Error)]
pub enum GitHubError {
    /// Ошибка HTTP-запроса.
    #[error(transparent)]
    Http(#[from] HttpError),
    /// Ответ GitHub API не удалось разобрать.
    #[error("непонятный ответ GitHub API: {0}")]
    InvalidResponse(String),
}

/// Нужная часть ответа `GET /repos/{owner}/{repo}/commits/{branch}`.
#[derive(Deserialize)]
struct CommitResponse {
    sha: String,
}

/// Публичный репозиторий GitHub.
pub struct GitHubRepository {
    owner: String,
    name: String,
}

impl GitHubRepository {
    /// Создаёт описание репозитория `owner/name`.
    pub fn new(owner: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            name: name.into(),
        }
    }

    /// Возвращает идентификатор последнего коммита основной ветки.
    pub fn latest_revision(&self, http: &HttpClient) -> Result<String, GitHubError> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/commits/{DEFAULT_BRANCH}",
            self.owner, self.name
        );
        let body = http.get_text(&url)?;
        let commit: CommitResponse = serde_json::from_str(&body)
            .map_err(|error| GitHubError::InvalidResponse(error.to_string()))?;
        validate_revision(&commit.sha)?;
        Ok(commit.sha)
    }

    /// Скачивает текст файла `path` в указанной ревизии.
    pub fn fetch_text(
        &self,
        http: &HttpClient,
        revision: &str,
        path: &str,
    ) -> Result<String, GitHubError> {
        Ok(http.get_text(&self.raw_url(revision, path)?)?)
    }

    /// Скачивает двоичный файл `path` (картинку) в указанной ревизии.
    pub fn fetch_bytes(
        &self,
        http: &HttpClient,
        revision: &str,
        path: &str,
    ) -> Result<Vec<u8>, GitHubError> {
        Ok(http.get_bytes(&self.raw_url(revision, path)?)?)
    }

    /// Адрес файла в «сыром» виде на raw.githubusercontent.com.
    /// Ревизия проверяется: она подставляется в адрес.
    fn raw_url(&self, revision: &str, path: &str) -> Result<String, GitHubError> {
        validate_revision(revision)?;
        Ok(format!(
            "https://raw.githubusercontent.com/{}/{}/{revision}/{path}",
            self.owner, self.name
        ))
    }
}

/// Проверяет, что ревизия — 40 шестнадцатеричных символов.
/// Значение подставляется в адрес, поэтому проверка обязательна.
fn validate_revision(revision: &str) -> Result<(), GitHubError> {
    let valid =
        revision.len() == REVISION_LENGTH && revision.chars().all(|c| c.is_ascii_hexdigit());
    if valid {
        Ok(())
    } else {
        Err(GitHubError::InvalidResponse(format!(
            "некорректная ревизия «{revision}»"
        )))
    }
}

// Тесты лежат рядом, в файле `github.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "github.tests.rs"]
mod tests;
