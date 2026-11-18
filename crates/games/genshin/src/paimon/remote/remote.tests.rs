//! Модульные тесты для `remote.rs`.
//!
//! Подключаются в `remote.rs` строками `#[cfg(test)] #[path = "remote.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

/// Минимальный набор файлов в формате Paimon.moe.
fn files() -> SourceFiles {
    [
        (
            source_files::BANNERS,
            "export const banners = { characters: [], weapons: [], chronicled: [], standard: [], beginners: [] };",
        ),
        (
            source_files::CHARACTERS,
            "export const characters = { mitya: { id: 'mitya', name: 'Mitya', rarity: 5 } };",
        ),
        (source_files::WEAPONS, "export const weaponList = {};"),
        (source_files::NAMES_RU, r#"{ "Mitya": "Митя" }"#),
    ]
    .into_iter()
    .map(|(path, text)| (path.to_owned(), text.to_owned()))
    .collect()
}

#[test]
fn source_lists_all_paimon_files() {
    let source = PaimonRemoteReference.source();

    assert_eq!(source.owner, "MadeBaruna");
    assert_eq!(source.repository, "paimon-moe");
    assert_eq!(source.files.len(), 4);
}

#[test]
fn builds_snapshot_from_downloaded_files() {
    let snapshot = PaimonRemoteReference
        .build(&files(), "rev", "2026-11-17 00:00:00")
        .unwrap();

    assert_eq!(snapshot.source_revision, "rev");
    assert_eq!(snapshot.items[0].name.ru, "Митя");
}

#[test]
fn missing_file_is_reported() {
    let mut incomplete = files();
    incomplete.remove(source_files::WEAPONS);

    let error = PaimonRemoteReference
        .build(&incomplete, "rev", "now")
        .unwrap_err();

    assert!(error.0.contains("weaponList.js"), "{error}");
}

#[test]
fn broken_file_is_reported() {
    let mut broken = files();
    broken.insert(
        source_files::BANNERS.to_owned(),
        "export const banners = {".to_owned(),
    );

    assert!(PaimonRemoteReference.build(&broken, "rev", "now").is_err());
}
