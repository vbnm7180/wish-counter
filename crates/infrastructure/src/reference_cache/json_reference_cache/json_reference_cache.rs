//! Кэш скачанных справочников в JSON-файлах (`data/catalog/`).
//!
//! Для каждой игры два файла:
//! - `<игра>.json` — скачанный справочник (тот же формат, что встроенный);
//! - `<игра>.state.json` — служебное состояние: время последней проверки.

use std::path::PathBuf;

use application::ports::{ReferenceCache, RepositoryError};
use chrono::NaiveDateTime;
use domain::game::GameId;
use domain::reference::ReferenceSnapshot;
use serde::{Deserialize, Serialize};

use crate::files::{is_safe_file_stem, read_json, unsafe_name_error, write_json_atomically};

/// Формат времени в файле состояния.
const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
/// Окончание имени файла справочника.
const SNAPSHOT_SUFFIX: &str = ".json";
/// Окончание имени файла состояния.
const STATE_SUFFIX: &str = ".state.json";

/// Содержимое файла `<игра>.state.json`.
#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheState {
    /// Время последней успешной проверки обновлений (UTC).
    last_checked_at: Option<String>,
}

/// Кэш справочников в папке `data/catalog/`.
pub struct JsonReferenceCache {
    directory: PathBuf,
}

impl JsonReferenceCache {
    /// Создаёт кэш в указанной папке (она должна существовать).
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    /// Путь к файлу игры с указанным окончанием имени.
    ///
    /// Идентификатор игры попадает в имя файла, поэтому проверяется:
    /// только `a-z`, `0-9`, `_`, `-`. Так через имя игры нельзя выйти
    /// за пределы папки кэша (например, `../../`).
    fn file_path(&self, game_id: &GameId, suffix: &str) -> Result<PathBuf, RepositoryError> {
        let id = game_id.as_str();
        if !is_safe_file_stem(id) {
            return Err(unsafe_name_error("игры", id));
        }
        Ok(self.directory.join(format!("{id}{suffix}")))
    }
}

impl ReferenceCache for JsonReferenceCache {
    /// Читает скачанный справочник; `None`, если файла нет.
    fn load_snapshot(
        &self,
        game_id: &GameId,
    ) -> Result<Option<ReferenceSnapshot>, RepositoryError> {
        read_json(&self.file_path(game_id, SNAPSHOT_SUFFIX)?)
    }

    /// Сохраняет скачанный справочник (атомарно).
    fn save_snapshot(
        &self,
        game_id: &GameId,
        snapshot: &ReferenceSnapshot,
    ) -> Result<(), RepositoryError> {
        write_json_atomically(&self.file_path(game_id, SNAPSHOT_SUFFIX)?, snapshot)
    }

    /// Читает время последней проверки. Испорченное значение считается отсутствующим:
    /// в худшем случае приложение просто проверит обновления ещё раз.
    fn last_checked_at(&self, game_id: &GameId) -> Result<Option<NaiveDateTime>, RepositoryError> {
        let state: Option<CacheState> = read_json(&self.file_path(game_id, STATE_SUFFIX)?)?;
        Ok(state
            .and_then(|state| state.last_checked_at)
            .and_then(|text| NaiveDateTime::parse_from_str(&text, TIMESTAMP_FORMAT).ok()))
    }

    /// Записывает время последней проверки.
    fn set_last_checked_at(
        &self,
        game_id: &GameId,
        at: NaiveDateTime,
    ) -> Result<(), RepositoryError> {
        let state = CacheState {
            last_checked_at: Some(at.format(TIMESTAMP_FORMAT).to_string()),
        };
        write_json_atomically(&self.file_path(game_id, STATE_SUFFIX)?, &state)
    }
}

// Тесты лежат рядом, в файле `json_reference_cache.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "json_reference_cache.tests.rs"]
mod tests;
