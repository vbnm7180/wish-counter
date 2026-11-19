//! Модульные тесты для `sqlite_profile_repository.rs`.
//!
//! Подключаются в `sqlite_profile_repository.rs` строками
//! `#[cfg(test)] #[path = "sqlite_profile_repository.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;
use crate::database::open_in_memory_database;

/// Хранилище на пустой базе в памяти.
fn repository() -> SqliteProfileRepository {
    SqliteProfileRepository::new(SharedConnection::new(open_in_memory_database().unwrap()))
}

/// Идентификатор Genshin для тестов.
fn genshin() -> GameId {
    GameId::new("genshin")
}

#[test]
fn region_is_absent_by_default() {
    assert_eq!(repository().server_region(&genshin()).unwrap(), None);
}

#[test]
fn saved_region_is_read_back() {
    let repository = repository();

    repository
        .set_server_region(&genshin(), ServerRegion::Europe)
        .unwrap();

    assert_eq!(
        repository.server_region(&genshin()).unwrap(),
        Some(ServerRegion::Europe)
    );
}

/// Повторное сохранение заменяет сервер, а не добавляет вторую запись.
#[test]
fn saving_again_replaces_region() {
    let repository = repository();

    repository
        .set_server_region(&genshin(), ServerRegion::Europe)
        .unwrap();
    repository
        .set_server_region(&genshin(), ServerRegion::America)
        .unwrap();

    assert_eq!(
        repository.server_region(&genshin()).unwrap(),
        Some(ServerRegion::America)
    );
}

#[test]
fn regions_of_different_games_are_independent() {
    let repository = repository();

    repository
        .set_server_region(&genshin(), ServerRegion::Asia)
        .unwrap();

    assert_eq!(repository.server_region(&GameId::new("hsr")).unwrap(), None);
}

/// База не даёт записать недопустимый сервер (ограничение CHECK в миграции).
#[test]
fn database_rejects_unknown_region_values() {
    let repository = repository();
    let result = repository.connection.lock().unwrap().execute(
        "INSERT INTO game_profiles (game_id, server_region) VALUES ('genshin', 'mars')",
        [],
    );
    assert!(result.is_err());
}

/// UID и сервер хранятся независимо: можно сохранить UID без сервера и наоборот.
#[test]
fn uid_and_region_are_independent() {
    let repository = repository();
    let uid = PlayerUid::parse("700000000").unwrap();

    repository.set_uid(&genshin(), &uid).unwrap();
    assert_eq!(repository.uid(&genshin()).unwrap(), Some(uid.clone()));
    assert_eq!(repository.server_region(&genshin()).unwrap(), None);

    repository
        .set_server_region(&genshin(), ServerRegion::Europe)
        .unwrap();
    assert_eq!(repository.uid(&genshin()).unwrap(), Some(uid));
    assert_eq!(
        repository.server_region(&genshin()).unwrap(),
        Some(ServerRegion::Europe)
    );
}

/// База не принимает UID с буквами (ограничение CHECK в миграции 0003).
#[test]
fn database_rejects_invalid_uid() {
    let repository = repository();
    let result = repository.connection.lock().unwrap().execute(
        "INSERT INTO game_profiles (game_id, uid) VALUES ('genshin', '7000000ab')",
        [],
    );
    assert!(result.is_err());
}
