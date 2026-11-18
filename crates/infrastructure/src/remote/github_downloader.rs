//! Загрузчик исходных данных справочника и картинок с GitHub.

use application::ports::{DownloadError, ImageDownloader, ReferenceDownloader};
use domain::reference::{ReferenceSourceSpec, SourceFiles};

use super::github::GitHubRepository;
use super::http_client::HttpClient;

/// Реализация [`ReferenceDownloader`] и [`ImageDownloader`] для публичных репозиториев GitHub.
pub struct GitHubReferenceDownloader {
    http: HttpClient,
}

impl GitHubReferenceDownloader {
    /// Создаёт загрузчик с безопасным HTTP-клиентом.
    pub fn new(http: HttpClient) -> Self {
        Self { http }
    }
}

impl ReferenceDownloader for GitHubReferenceDownloader {
    /// Последний коммит основной ветки репозитория.
    fn latest_revision(&self, source: &ReferenceSourceSpec) -> Result<String, DownloadError> {
        repository(source)
            .latest_revision(&self.http)
            .map_err(|error| DownloadError(error.to_string()))
    }

    /// Скачивает все файлы источника в указанной ревизии.
    ///
    /// `collect()` в `Result<HashMap, _>` останавливается на первой ошибке.
    fn download(
        &self,
        source: &ReferenceSourceSpec,
        revision: &str,
    ) -> Result<SourceFiles, DownloadError> {
        let repository = repository(source);
        source
            .files
            .iter()
            .map(|path| {
                let text = repository
                    .fetch_text(&self.http, revision, path)
                    .map_err(|error| DownloadError(error.to_string()))?;
                Ok((path.clone(), text))
            })
            .collect()
    }
}

impl ImageDownloader for GitHubReferenceDownloader {
    /// Скачивает картинку из репозитория источника.
    fn download_image(
        &self,
        source: &ReferenceSourceSpec,
        revision: &str,
        path: &str,
    ) -> Result<Vec<u8>, DownloadError> {
        repository(source)
            .fetch_bytes(&self.http, revision, path)
            .map_err(|error| DownloadError(error.to_string()))
    }
}

/// Описание репозитория GitHub по описанию источника.
fn repository(source: &ReferenceSourceSpec) -> GitHubRepository {
    GitHubRepository::new(&source.owner, &source.repository)
}
