//! Миграции схемы базы данных.
//!
//! **Миграция** — SQL-скрипт, который переводит схему базы из одной версии
//! в следующую. Номер текущей версии SQLite хранит в самом файле базы
//! (`PRAGMA user_version`). При запуске применяются только те миграции,
//! номер которых больше сохранённого.
//!
//! Как добавить миграцию:
//! 1. создать `sql/000N_описание.sql`;
//! 2. добавить его в конец [`MIGRATIONS`];
//! 3. **никогда не менять** уже выпущенные миграции — у пользователей они
//!    уже применены, и изменения до них не дойдут.

use rusqlite::{Connection, Transaction};

use super::DatabaseError;

/// Все миграции по порядку. Версия схемы = количество применённых миграций.
///
/// `include_str!` встраивает содержимое файла в программу при компиляции,
/// поэтому SQL-файлы не нужно поставлять рядом с `.exe`.
const MIGRATIONS: &[&str] = &[
    include_str!("sql/0001_create_wishes.sql"),
    include_str!("sql/0002_create_game_profiles.sql"),
];

/// Применяет к базе все недостающие миграции.
///
/// Каждая миграция выполняется в отдельной транзакции вместе с обновлением
/// номера версии: если скрипт упадёт, база останется в прежней версии.
pub fn apply_migrations(connection: &mut Connection) -> Result<(), DatabaseError> {
    let current = schema_version(connection)?;
    let latest = latest_version();
    if current > latest {
        return Err(DatabaseError::NewerSchema {
            found: current,
            supported: latest,
        });
    }
    for (index, script) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        let version = index as u32 + 1;
        let transaction = connection.transaction()?;
        apply_migration(&transaction, script, version)?;
        transaction.commit()?;
        tracing::info!(version, "применена миграция базы данных");
    }
    Ok(())
}

/// Версия схемы, которую поддерживает эта сборка приложения.
pub fn latest_version() -> u32 {
    MIGRATIONS.len() as u32
}

/// Читает текущую версию схемы из файла базы.
pub fn schema_version(connection: &Connection) -> Result<u32, DatabaseError> {
    Ok(connection.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

/// Выполняет один скрипт миграции и записывает новый номер версии.
fn apply_migration(
    transaction: &Transaction<'_>,
    script: &str,
    version: u32,
) -> Result<(), DatabaseError> {
    transaction.execute_batch(script)?;
    transaction.pragma_update(None, "user_version", version)?;
    Ok(())
}

// Тесты лежат рядом, в файле `migrations.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "migrations.tests.rs"]
mod tests;
