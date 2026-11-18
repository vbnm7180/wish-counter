//! Модульные тесты для `file_image_cache.rs`.
//!
//! Подключаются в `file_image_cache.rs` строками
//! `#[cfg(test)] #[path = "file_image_cache.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

/// Кэш во временной папке.
fn cache() -> (tempfile::TempDir, FileImageCache) {
    let temp = tempfile::tempdir().unwrap();
    let cache = FileImageCache::new(temp.path());
    (temp, cache)
}

#[test]
fn missing_image_reads_as_none() {
    let (_temp, cache) = cache();
    let result = cache.load(&GameId::new("genshin"), &ItemId::new("hu_tao"));
    assert_eq!(result.unwrap(), None);
}

#[test]
fn saved_image_is_stored_per_game() {
    let (temp, cache) = cache();
    let genshin = GameId::new("genshin");
    let hu_tao = ItemId::new("hu_tao");

    cache.save(&genshin, &hu_tao, &[1, 2, 3]).unwrap();

    assert_eq!(cache.load(&genshin, &hu_tao).unwrap(), Some(vec![1, 2, 3]));
    assert!(temp.path().join("genshin").join("hu_tao.png").is_file());
}

/// Ни идентификатор игры, ни идентификатор предмета не могут увести путь из папки кэша.
#[test]
fn unsafe_ids_are_rejected() {
    let (_temp, cache) = cache();

    assert!(
        cache
            .load(&GameId::new(".."), &ItemId::new("hu_tao"))
            .is_err()
    );
    assert!(
        cache
            .load(&GameId::new("genshin"), &ItemId::new("../secret"))
            .is_err()
    );
    assert!(
        cache
            .save(&GameId::new("genshin"), &ItemId::new("a/b"), &[1])
            .is_err()
    );
}
