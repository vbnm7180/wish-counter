//! Кэш картинок предметов в файлах: `data/cache/images/<игра>/<предмет>.png`.

use std::fs;
use std::path::PathBuf;

use application::ports::{ImageCache, RepositoryError};
use domain::game::GameId;
use domain::wish::ItemId;

use crate::files::{is_safe_file_stem, read_bytes, unsafe_name_error, write_bytes_atomically};

/// Расширение файлов картинок.
const IMAGE_EXTENSION: &str = "png";

/// Кэш картинок в папке на диске.
pub struct FileImageCache {
    directory: PathBuf,
}

impl FileImageCache {
    /// Создаёт кэш в указанной папке (она должна существовать).
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    /// Папка картинок игры.
    fn game_dir(&self, game_id: &GameId) -> Result<PathBuf, RepositoryError> {
        let game = game_id.as_str();
        if !is_safe_file_stem(game) {
            return Err(unsafe_name_error("игры", game));
        }
        Ok(self.directory.join(game))
    }

    /// Путь к картинке предмета. Оба идентификатора проверяются,
    /// чтобы через них нельзя было выйти за пределы папки кэша.
    fn image_path(&self, game_id: &GameId, item_id: &ItemId) -> Result<PathBuf, RepositoryError> {
        let item = item_id.as_str();
        if !is_safe_file_stem(item) {
            return Err(unsafe_name_error("предмета", item));
        }
        Ok(self
            .game_dir(game_id)?
            .join(format!("{item}.{IMAGE_EXTENSION}")))
    }
}

impl ImageCache for FileImageCache {
    /// Читает картинку; `None`, если её нет.
    fn load(&self, game_id: &GameId, item_id: &ItemId) -> Result<Option<Vec<u8>>, RepositoryError> {
        read_bytes(&self.image_path(game_id, item_id)?)
    }

    /// Сохраняет картинку (папка игры создаётся при первой записи).
    fn save(
        &self,
        game_id: &GameId,
        item_id: &ItemId,
        bytes: &[u8],
    ) -> Result<(), RepositoryError> {
        let path = self.image_path(game_id, item_id)?;
        fs::create_dir_all(self.game_dir(game_id)?)
            .map_err(|error| RepositoryError::Io(error.to_string()))?;
        write_bytes_atomically(&path, bytes)
    }
}

// Тесты лежат рядом, в файле `file_image_cache.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "file_image_cache.tests.rs"]
mod tests;
