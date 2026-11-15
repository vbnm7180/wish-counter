//! Хранилище профилей игр в базе SQLite (таблица `game_profiles`, миграция 0002).

use application::ports::{ProfileRepository, RepositoryError};
use domain::game::GameId;
use domain::region::ServerRegion;
use rusqlite::{OptionalExtension, params};

use crate::database::SharedConnection;

/// Сервер игрока для игры.
const SELECT_REGION_SQL: &str = "SELECT server_region FROM game_profiles WHERE game_id = ?1";

/// Вставка или замена сервера (UPSERT: если профиль уже есть — обновить).
const UPSERT_REGION_SQL: &str = "
    INSERT INTO game_profiles (game_id, server_region) VALUES (?1, ?2)
    ON CONFLICT (game_id) DO UPDATE SET server_region = excluded.server_region";

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
        let connection = self.connection.lock()?;
        // `.optional()` превращает «строк нет» в `Ok(None)` вместо ошибки.
        let stored: Option<String> = connection
            .query_row(SELECT_REGION_SQL, [game_id.as_str()], |row| row.get(0))
            .optional()
            .map_err(sql_error)?;
        stored
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
