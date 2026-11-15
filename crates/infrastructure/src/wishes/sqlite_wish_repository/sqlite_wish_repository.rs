//! Хранилище истории молитв в базе SQLite (таблица `wishes`).

use std::sync::{Mutex, MutexGuard};

use application::ports::{RepositoryError, WishRepository};
use domain::game::GameId;
use domain::wish::{BannerCode, ItemId, ItemKind, Rarity, Wish, WishId, WishTime};
use rusqlite::{Connection, Row, params};

/// Все молитвы игры в хронологическом порядке. При равном времени
/// порядок задаёт `row_id` — порядок добавления (см. миграцию 0001).
const SELECT_WISHES_SQL: &str = "
    SELECT wish_id, banner_code, item_id, item_kind, rarity, wish_time
    FROM wishes
    WHERE game_id = ?1
    ORDER BY wish_time, row_id";

/// Добавление молитвы. `OR IGNORE` молча пропускает молитву, если пара
/// (game_id, wish_id) уже есть в таблице (ограничение UNIQUE).
const INSERT_WISH_SQL: &str = "
    INSERT OR IGNORE INTO wishes (game_id, wish_id, banner_code, item_id, item_kind, rarity, wish_time)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)";

/// Значение столбца `item_kind` для персонажа.
const ITEM_KIND_CHARACTER: &str = "character";
/// Значение столбца `item_kind` для оружия.
const ITEM_KIND_WEAPON: &str = "weapon";

/// Молитва в том виде, в котором она лежит в таблице (ещё без проверок).
///
/// Сначала строка читается «как есть», затем [`StoredWish::into_wish`]
/// проверяет значения. Так ошибка в данных превращается в понятную
/// `RepositoryError::Corrupted`, а не в панику.
struct StoredWish {
    wish_id: String,
    banner_code: u32,
    item_id: String,
    item_kind: String,
    rarity: u8,
    wish_time: String,
}

impl StoredWish {
    /// Читает поля из строки результата запроса [`SELECT_WISHES_SQL`].
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            wish_id: row.get(0)?,
            banner_code: row.get(1)?,
            item_id: row.get(2)?,
            item_kind: row.get(3)?,
            rarity: row.get(4)?,
            wish_time: row.get(5)?,
        })
    }

    /// Проверяет значения и превращает строку таблицы в доменную молитву.
    fn into_wish(self) -> Result<Wish, RepositoryError> {
        Ok(Wish {
            item_kind: item_kind_from_sql(&self.item_kind)?,
            rarity: Rarity::try_from(self.rarity).map_err(corrupted)?,
            time: WishTime::parse(self.wish_time).map_err(corrupted)?,
            id: WishId::new(self.wish_id),
            banner_code: BannerCode::new(self.banner_code),
            item_id: ItemId::new(self.item_id),
        })
    }
}

/// Хранилище молитв на SQLite.
///
/// `rusqlite::Connection` нельзя одновременно использовать из нескольких
/// потоков, а Tauri вызывает команды в разных потоках. `Mutex` пропускает
/// к соединению только один поток за раз — так хранилище становится `Sync`.
pub struct SqliteWishRepository {
    connection: Mutex<Connection>,
}

impl SqliteWishRepository {
    /// Создаёт хранилище поверх открытого соединения
    /// (см. `database::open_database`).
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Mutex::new(connection),
        }
    }

    /// Захватывает соединение для одного запроса.
    ///
    /// `Mutex` «отравляется» (poisoned), если поток упал, удерживая его.
    /// Тогда возвращаем ошибку, а не падаем следом.
    fn lock(&self) -> Result<MutexGuard<'_, Connection>, RepositoryError> {
        self.connection
            .lock()
            .map_err(|_| RepositoryError::Io("соединение с базой недоступно после сбоя".into()))
    }
}

impl WishRepository for SqliteWishRepository {
    /// Читает молитвы игры от старых к новым.
    fn list_wishes(&self, game_id: &GameId) -> Result<Vec<Wish>, RepositoryError> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare_cached(SELECT_WISHES_SQL)
            .map_err(sql_error)?;
        let rows = statement
            .query_map([game_id.as_str()], StoredWish::from_row)
            .map_err(sql_error)?;
        rows.map(|row| row.map_err(sql_error)?.into_wish())
            .collect()
    }

    /// Добавляет молитвы в одной транзакции и считает новые.
    fn add_wishes(&self, game_id: &GameId, wishes: &[Wish]) -> Result<usize, RepositoryError> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(sql_error)?;
        let mut inserted = 0;
        {
            // Подготовленный запрос компилируется один раз и выполняется для каждой молитвы.
            // Блок `{ }` нужен, чтобы `statement` освободил `transaction` до `commit`.
            let mut statement = transaction
                .prepare_cached(INSERT_WISH_SQL)
                .map_err(sql_error)?;
            for wish in wishes {
                inserted += statement
                    .execute(params![
                        game_id.as_str(),
                        wish.id.as_str(),
                        wish.banner_code.value(),
                        wish.item_id.as_str(),
                        item_kind_to_sql(wish.item_kind),
                        u8::from(wish.rarity),
                        wish.time.as_str(),
                    ])
                    .map_err(sql_error)?;
            }
        }
        transaction.commit().map_err(sql_error)?;
        tracing::info!(game = %game_id, received = wishes.len(), inserted, "молитвы сохранены");
        Ok(inserted)
    }
}

/// Превращает тип предмета в значение столбца `item_kind`.
fn item_kind_to_sql(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Character => ITEM_KIND_CHARACTER,
        ItemKind::Weapon => ITEM_KIND_WEAPON,
    }
}

/// Читает тип предмета из столбца `item_kind`.
fn item_kind_from_sql(value: &str) -> Result<ItemKind, RepositoryError> {
    match value {
        ITEM_KIND_CHARACTER => Ok(ItemKind::Character),
        ITEM_KIND_WEAPON => Ok(ItemKind::Weapon),
        other => Err(RepositoryError::Corrupted(format!(
            "неизвестный тип предмета «{other}»"
        ))),
    }
}

/// Превращает ошибку SQLite в ошибку хранилища.
fn sql_error(error: rusqlite::Error) -> RepositoryError {
    RepositoryError::Io(error.to_string())
}

/// Превращает ошибку проверки значения в «повреждённые данные».
fn corrupted(error: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::Corrupted(error.to_string())
}

// Тесты лежат рядом, в файле `sqlite_wish_repository.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "sqlite_wish_repository.tests.rs"]
mod tests;
