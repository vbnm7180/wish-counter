//! Хранилище профилей игр в базе SQLite (таблица `game_profiles`, миграции 0002 и 0003).

use application::ports::{ProfileRepository, RepositoryError};
use domain::game::GameId;
use domain::profile::PlayerUid;
use domain::region::ServerRegion;
use rusqlite::{OptionalExtension, params};

use crate::database::SharedConnection;

/// Сервер игрока для игры.
const SELECT_REGION_SQL: &str = "SELECT server_region FROM game_profiles WHERE game_id = ?1";

/// Вставка или замена сервера (UPSERT: если профиль уже есть — обновить).
const UPSERT_REGION_SQL: &str = "
    INSERT INTO game_profiles (game_id, server_region) VALUES (?1, ?2)
    ON CONFLICT (game_id) DO UPDATE SET server_region = excluded.server_region";

/// UID игрока для игры.
const SELECT_UID_SQL: &str = "SELECT uid FROM game_profiles WHERE game_id = ?1";

/// Вставка или замена UID.
const UPSERT_UID_SQL: &str = "
    INSERT INTO game_profiles (game_id, uid) VALUES (?1, ?2)
    ON CONFLICT (game_id) DO UPDATE SET uid = excluded.uid";

/// Хранилище профилей игр на SQLite.
pub struct SqliteProfileRepository {
    connection: SharedConnection,
}

impl SqliteProfileRepository {
    /// Создаёт хранилище поверх общего соединения с базой.
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }
}

impl ProfileRepository for SqliteProfileRepository {
    /// Читает сервер игрока; неизвестное значение в базе — ошибка `Corrupted`.
    fn server_region(&self, game_id: &GameId) -> Result<Option<ServerRegion>, RepositoryError> {
        self.read_text(SELECT_REGION_SQL, game_id)?
            .map(|value| {
                ServerRegion::parse(&value).ok_or_else(|| {
                    RepositoryError::Corrupted(format!("неизвестный сервер «{value}»"))
                })
            })
            .transpose()
    }

    /// Сохраняет сервер игрока.
    fn set_server_region(
        &self,
        game_id: &GameId,
        region: ServerRegion,
    ) -> Result<(), RepositoryError> {
        self.connection
            .lock()?
            .execute(
                UPSERT_REGION_SQL,
                params![game_id.as_str(), region.as_str()],
            )
            .map_err(sql_error)?;
        tracing::info!(game = %game_id, %region, "сервер игрока сохранён");
        Ok(())
    }

    /// Читает UID игрока; некорректное значение в базе — ошибка `Corrupted`.
    fn uid(&self, game_id: &GameId) -> Result<Option<PlayerUid>, RepositoryError> {
        self.read_text(SELECT_UID_SQL, game_id)?
            .map(|value| {
                PlayerUid::parse(&value)
                    .map_err(|error| RepositoryError::Corrupted(error.to_string()))
            })
            .transpose()
    }

    /// Сохраняет UID игрока. Сам UID в лог не пишется — только факт сохранения.
    fn set_uid(&self, game_id: &GameId, uid: &PlayerUid) -> Result<(), RepositoryError> {
        self.connection
            .lock()?
            .execute(UPSERT_UID_SQL, params![game_id.as_str(), uid.as_str()])
            .map_err(sql_error)?;
        tracing::info!(game = %game_id, "UID игрока сохранён");
        Ok(())
    }

    fn delete_profile(&self, game_id: &GameId) -> Result<(), RepositoryError> {
        self.connection
            .lock()?
            .execute("DELETE FROM game_profiles WHERE game_id = ?1", params![game_id.as_str()])
            .map_err(sql_error)?;
        tracing::info!(game = %game_id, "профиль игры удалён");
        Ok(())
    }
}

impl SqliteProfileRepository {
    /// Читает один текстовый столбец профиля игры; `None` — нет профиля или значение пустое.
    fn read_text(&self, sql: &str, game_id: &GameId) -> Result<Option<String>, RepositoryError> {
        let connection = self.connection.lock()?;
        // `.optional()` превращает «строк нет» в `Ok(None)`; `flatten` — NULL в столбце тоже `None`.
        let stored: Option<Option<String>> = connection
            .query_row(sql, [game_id.as_str()], |row| row.get(0))
            .optional()
            .map_err(sql_error)?;
        Ok(stored.flatten())
    }
}

/// Превращает ошибку SQLite в ошибку хранилища.
fn sql_error(error: rusqlite::Error) -> RepositoryError {
    RepositoryError::Io(error.to_string())
}

// Тесты лежат рядом, в файле `sqlite_profile_repository.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "sqlite_profile_repository.tests.rs"]
mod tests;
