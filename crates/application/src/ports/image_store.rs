//! Интерфейсы для картинок предметов: локальный кэш и загрузчик.

use domain::game::GameId;
use domain::reference::ReferenceSourceSpec;
use domain::wish::ItemId;

use super::{DownloadError, RepositoryError};

/// Локальный кэш картинок (в portable-режиме — `data/cache/images/`).
///
/// Картинки хранятся на диске постоянно: после первого показа
/// они доступны без интернета.
pub trait ImageCache: Send + Sync {
    /// Читает картинку предмета; `None`, если её ещё нет в кэше.
    fn load(&self, game_id: &GameId, item_id: &ItemId) -> Result<Option<Vec<u8>>, RepositoryError>;

    /// Сохраняет картинку предмета.
    fn save(&self, game_id: &GameId, item_id: &ItemId, bytes: &[u8])
    -> Result<(), RepositoryError>;
}

/// Скачивает картинку из репозитория источника справочника.
pub trait ImageDownloader: Send + Sync {
    /// Скачивает файл `path` ревизии `revision` репозитория `source`.
    fn download_image(
        &self,
        source: &ReferenceSourceSpec,
        revision: &str,
        path: &str,
    ) -> Result<Vec<u8>, DownloadError>;
}
