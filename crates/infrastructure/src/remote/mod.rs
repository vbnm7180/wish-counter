//! Доступ к сети: безопасный HTTP-клиент, чтение файлов с GitHub
//! и загрузчик справочников.

// Код модуля лежит в папке модуля вместе с тестами: github/github.rs
#[path = "github/github.rs"]
pub mod github;
mod github_downloader;
/// HTTP-клиент для API игры (HoYoverse).
pub mod game_api_client;
// Код модуля лежит в папке модуля вместе с тестами: http_client/http_client.rs
#[path = "http_client/http_client.rs"]
pub mod http_client;

pub use game_api_client::GameApiHttpClient;
pub use github::{GitHubError, GitHubRepository};
pub use github_downloader::GitHubReferenceDownloader;
pub use http_client::{HttpClient, HttpError};
