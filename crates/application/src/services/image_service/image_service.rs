//! Картинки предметов: из локального кэша или, при первом запросе, из сети.
//!
//! Порядок:
//! 1. картинка есть в кэше — отдать её (работает без интернета);
//! 2. иначе найти предмет в справочнике игры — скачивать можно только картинки
//!    известных предметов, произвольный путь запросить нельзя;
//! 3. скачать по пути, который знает модуль игры (`RemoteReference::image_path`),
//!    проверить, что это PNG, сохранить в кэш и отдать.
//!
//! Если картинку получить не удалось (нет сети, нет файла у источника),
//! предмет запоминается до конца работы приложения, чтобы не повторять
//! запросы при каждой перерисовке. Интерфейс показывает заглушку.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, PoisonError};

use domain::game::GameId;
use domain::wish::ItemId;
use thiserror::Error;

use super::{GameCatalog, ReferenceService};
use crate::ports::{ImageCache, ImageDownloader, RepositoryError};

/// Первые 8 байт любого PNG-файла (сигнатура формата).
const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Ошибка получения картинки.
#[derive(Debug, Error)]
pub enum ImageError {
    /// Игра не зарегистрирована.
    #[error("игра «{0}» не поддерживается")]
    UnknownGame(GameId),
    /// Не удалось прочитать или записать кэш.
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Отдаёт картинки предметов, скачивая и кэшируя их при первом запросе.
pub struct ImageService {
    catalog: Arc<GameCatalog>,
    references: Arc<ReferenceService>,
    cache: Arc<dyn ImageCache>,
    downloader: Arc<dyn ImageDownloader>,
    /// Картинки, которые в этом сеансе получить не удалось.
    unavailable: Mutex<HashSet<(GameId, ItemId)>>,
}

impl ImageService {
    /// Создаёт сервис картинок.
    pub fn new(
        catalog: Arc<GameCatalog>,
        references: Arc<ReferenceService>,
        cache: Arc<dyn ImageCache>,
        downloader: Arc<dyn ImageDownloader>,
    ) -> Self {
        Self {
            catalog,
            references,
            cache,
            downloader,
            unavailable: Mutex::new(HashSet::new()),
        }
    }

    /// Возвращает PNG-картинку предмета; `None`, если её нет и получить нельзя.
    pub fn image(&self, game_id: &GameId, item_id: &ItemId) -> Result<Option<Vec<u8>>, ImageError> {
        if let Some(bytes) = self.cache.load(game_id, item_id)? {
            return Ok(Some(bytes));
        }
        if self.is_unavailable(game_id, item_id) {
            return Ok(None);
        }
        let downloaded = self.download(game_id, item_id)?;
        match downloaded {
            Some(bytes) => {
                self.cache.save(game_id, item_id, &bytes)?;
                Ok(Some(bytes))
            }
            None => {
                self.mark_unavailable(game_id, item_id);
                Ok(None)
            }
        }
    }

    /// Скачивает в кэш картинки указанных предметов, которых там ещё нет
    /// (после импорта истории — чтобы потом всё показывалось без интернета).
    /// Возвращает, сколько картинок теперь есть в кэше. Ошибки отдельных
    /// картинок не прерывают докачку остальных.
    pub fn prefetch(&self, game_id: &GameId, item_ids: &[ItemId]) -> usize {
        item_ids
            .iter()
            .filter(|item_id| matches!(self.image(game_id, item_id), Ok(Some(_))))
            .count()
    }

    /// Возвращает иконку игры (встроенную в модуль игры); `None`, если её нет.
    pub fn game_icon(&self, game_id: &GameId) -> Result<Option<Vec<u8>>, ImageError> {
        let module = self
            .catalog
            .find(game_id)
            .ok_or_else(|| ImageError::UnknownGame(game_id.clone()))?;
        Ok(module.icon_png().map(<[u8]>::to_vec))
    }

    /// Скачивает картинку известного предмета. `None` — скачать не удалось
    /// или это не PNG (причина пишется в лог).
    fn download(&self, game_id: &GameId, item_id: &ItemId) -> Result<Option<Vec<u8>>, ImageError> {
        let module = self
            .catalog
            .find(game_id)
            .ok_or_else(|| ImageError::UnknownGame(game_id.clone()))?;
        let Some(remote) = module.remote_reference() else {
            return Ok(None);
        };
        let reference = self
            .references
            .reference(game_id)
            .map_err(|_| ImageError::UnknownGame(game_id.clone()))?;
        let Some(path) = reference
            .item(item_id)
            .and_then(|item| remote.image_path(item))
        else {
            return Ok(None);
        };
        let revision = &reference.snapshot().source_revision;
        match self
            .downloader
            .download_image(&remote.source(), revision, &path)
        {
            Ok(bytes) if is_png(&bytes) => Ok(Some(bytes)),
            Ok(_) => {
                tracing::warn!(game = %game_id, item = %item_id, "скачанный файл — не PNG");
                Ok(None)
            }
            Err(error) => {
                tracing::warn!(game = %game_id, item = %item_id, %error, "картинка не скачана");
                Ok(None)
            }
        }
    }

    /// Не удавалось ли уже получить эту картинку в этом сеансе.
    fn is_unavailable(&self, game_id: &GameId, item_id: &ItemId) -> bool {
        self.unavailable
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .contains(&(game_id.clone(), item_id.clone()))
    }

    /// Запоминает, что картинку получить не удалось.
    fn mark_unavailable(&self, game_id: &GameId, item_id: &ItemId) {
        self.unavailable
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert((game_id.clone(), item_id.clone()));
    }
}

/// Проверяет, что байты — PNG-файл (по сигнатуре в начале).
fn is_png(bytes: &[u8]) -> bool {
    bytes.starts_with(&PNG_SIGNATURE)
}

// Тесты лежат рядом, в файле `image_service.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "image_service.tests.rs"]
mod tests;
