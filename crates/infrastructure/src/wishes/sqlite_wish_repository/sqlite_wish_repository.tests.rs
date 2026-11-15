//! Модульные тесты для `sqlite_wish_repository.rs`.
//!
//! Подключаются в `sqlite_wish_repository.rs` строками
//! `#[cfg(test)] #[path = "sqlite_wish_repository.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.
//!
//! Тесты работают с настоящей SQLite в оперативной памяти.

use super::*;
use crate::database::{SharedConnection, open_in_memory_database};

/// Хранилище на пустой базе в памяти.
fn repository() -> SqliteWishRepository {
    SqliteWishRepository::new(SharedConnection::new(open_in_memory_database().unwrap()))
}

/// Идентификатор Genshin для тестов.
fn genshin() -> GameId {
    GameId::new("genshin")
}

/// Создаёт тестовую молитву.
fn wish(id: &str, time: &str, rarity: Rarity, kind: ItemKind) -> Wish {
    Wish {
        id: WishId::new(id),
        banner_code: BannerCode::new(301),
        item_id: ItemId::new(format!("item_{id}")),
        item_kind: kind,
        rarity,
        time: WishTime::parse(time).unwrap(),
    }
}

/// Обычная молитва с 3★ оружием.
fn three(id: &str, time: &str) -> Wish {
    wish(id, time, Rarity::Three, ItemKind::Weapon)
}

/// Идентификаторы молитв игры в порядке, который вернуло хранилище.
fn stored_ids(repository: &SqliteWishRepository, game: &GameId) -> Vec<String> {
    repository
        .list_wishes(game)
        .unwrap()
        .into_iter()
        .map(|wish| wish.id.as_str().to_owned())
        .collect()
}

#[test]
fn empty_database_returns_no_wishes() {
    assert!(repository().list_wishes(&genshin()).unwrap().is_empty());
}

/// Все поля молитвы сохраняются и читаются без изменений.
#[test]
fn saved_wish_is_read_back_unchanged() {
    let repository = repository();
    let original = wish(
        "1700000000000000001",
        "2026-11-15 18:04:59",
        Rarity::Five,
        ItemKind::Character,
    );

    repository
        .add_wishes(&genshin(), std::slice::from_ref(&original))
        .unwrap();

    assert_eq!(repository.list_wishes(&genshin()).unwrap(), vec![original]);
}

/// Молитвы возвращаются по времени, а при равном времени — в порядке добавления.
#[test]
fn wishes_are_returned_chronologically() {
    let repository = repository();
    repository
        .add_wishes(
            &genshin(),
            &[
                three("ten-1", "2026-02-01 10:00:00"),
                three("ten-2", "2026-02-01 10:00:00"),
            ],
        )
        .unwrap();
    repository
        .add_wishes(&genshin(), &[three("older", "2026-01-01 00:00:00")])
        .unwrap();

    assert_eq!(
        stored_ids(&repository, &genshin()),
        ["older", "ten-1", "ten-2"]
    );
}

/// Повторное добавление тех же молитв ничего не дублирует.
#[test]
fn duplicate_wishes_are_skipped() {
    let repository = repository();
    let wishes = [
        three("1", "2026-01-01 00:00:00"),
        three("2", "2026-01-01 00:00:01"),
    ];

    let first = repository.add_wishes(&genshin(), &wishes).unwrap();
    let second = repository.add_wishes(&genshin(), &wishes).unwrap();

    assert_eq!((first, second), (2, 0));
    assert_eq!(stored_ids(&repository, &genshin()).len(), 2);
}

/// Одинаковый идентификатор в разных играх — разные молитвы.
#[test]
fn games_are_stored_separately() {
    let repository = repository();
    let hsr = GameId::new("hsr");

    repository
        .add_wishes(&genshin(), &[three("1", "2026-01-01 00:00:00")])
        .unwrap();
    repository
        .add_wishes(&hsr, &[three("1", "2026-01-01 00:00:00")])
        .unwrap();

    assert_eq!(stored_ids(&repository, &genshin()), ["1"]);
    assert_eq!(stored_ids(&repository, &hsr), ["1"]);
}

/// Испорченное значение в таблице даёт понятную ошибку, а не панику.
#[test]
fn invalid_time_in_table_is_reported_as_corrupted() {
    let repository = repository();
    repository
        .connection
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO wishes (game_id, wish_id, banner_code, item_id, item_kind, rarity, wish_time)
             VALUES ('genshin', '1', 301, 'sword', 'weapon', 3, 'вчера')",
            [],
        )
        .unwrap();

    assert!(matches!(
        repository.list_wishes(&genshin()),
        Err(RepositoryError::Corrupted(_))
    ));
}

#[test]
fn item_kind_roundtrips_through_sql_value() {
    for kind in [ItemKind::Character, ItemKind::Weapon] {
        assert_eq!(item_kind_from_sql(item_kind_to_sql(kind)).unwrap(), kind);
    }
    assert!(item_kind_from_sql("pet").is_err());
}
