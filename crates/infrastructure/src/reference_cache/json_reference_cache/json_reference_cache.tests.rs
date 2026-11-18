//! Модульные тесты для `json_reference_cache.rs`.
//!
//! Подключаются в `json_reference_cache.rs` строками
//! `#[cfg(test)] #[path = "json_reference_cache.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

/// Кэш во временной папке. `TempDir` удаляет папку, когда выходит из области видимости.
fn cache() -> (tempfile::TempDir, JsonReferenceCache) {
    let temp = tempfile::tempdir().unwrap();
    let cache = JsonReferenceCache::new(temp.path());
    (temp, cache)
}

/// Маленький справочник для записи.
fn snapshot() -> ReferenceSnapshot {
    ReferenceSnapshot {
        source_revision: "rev".into(),
        generated_at: "2026-11-17 10:00:00".into(),
        items: vec![],
        banners: vec![],
    }
}

/// Время для тестов.
fn time(text: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(text, TIMESTAMP_FORMAT).unwrap()
}

#[test]
fn empty_cache_has_nothing() {
    let (_temp, cache) = cache();
    let genshin = GameId::new("genshin");

    assert!(cache.load_snapshot(&genshin).unwrap().is_none());
    assert!(cache.last_checked_at(&genshin).unwrap().is_none());
}

#[test]
fn snapshot_is_saved_and_loaded() {
    let (temp, cache) = cache();
    let genshin = GameId::new("genshin");

    cache.save_snapshot(&genshin, &snapshot()).unwrap();

    assert_eq!(cache.load_snapshot(&genshin).unwrap(), Some(snapshot()));
    assert!(temp.path().join("genshin.json").is_file());
}

#[test]
fn last_check_time_is_saved_and_loaded() {
    let (_temp, cache) = cache();
    let genshin = GameId::new("genshin");

    cache
        .set_last_checked_at(&genshin, time("2026-11-17 10:00:00"))
        .unwrap();

    assert_eq!(
        cache.last_checked_at(&genshin).unwrap(),
        Some(time("2026-11-17 10:00:00"))
    );
}

/// Идентификатор игры с `..` или `/` не может стать путём за пределами папки.
#[test]
fn unsafe_game_ids_are_rejected() {
    let (_temp, cache) = cache();
    for id in ["../escape", "a/b", "C:", ""] {
        assert!(cache.load_snapshot(&GameId::new(id)).is_err(), "{id}");
    }
}

/// Испорченное время в файле состояния — «проверок не было», а не ошибка.
#[test]
fn corrupted_check_time_reads_as_none() {
    let (temp, cache) = cache();
    let state = "{ \"last_checked_at\": \"yesterday\" }";
    std::fs::write(temp.path().join("genshin.state.json"), state).unwrap();

    assert!(
        cache
            .last_checked_at(&GameId::new("genshin"))
            .unwrap()
            .is_none()
    );
}
