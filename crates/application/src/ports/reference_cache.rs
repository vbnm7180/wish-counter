//! Интерфейс хранилища скачанных справочников.

use chrono::NaiveDateTime;
use domain::game::GameId;
use domain::reference::ReferenceSnapshot;

use super::RepositoryError;

/// Хранит скачанные обновления справочников и время последней проверки.
///
/// Встроенный в программу справочник сюда не попадает: здесь только то,
/// что приложение скачало само (в portable-режиме — `data/catalog/`).
pub trait ReferenceCache: Send + Sync {
    /// Читает скачанный справочник игры; `None`, если его нет.
    fn load_snapshot(&self, game_id: &GameId)
    -> Result<Option<ReferenceSnapshot>, RepositoryError>;

    /// Сохраняет скачанный справочник игры (заменяя прежний).
    fn save_snapshot(
        &self,
        game_id: &GameId,
        snapshot: &ReferenceSnapshot,
    ) -> Result<(), RepositoryError>;

    /// Когда последний раз **успешно** проверялось обновление (UTC); `None` — ни разу.
    fn last_checked_at(&self, game_id: &GameId) -> Result<Option<NaiveDateTime>, RepositoryError>;

    /// Запоминает время успешной проверки (UTC).
    fn set_last_checked_at(
        &self,
        game_id: &GameId,
        at: NaiveDateTime,
    ) -> Result<(), RepositoryError>;
}
