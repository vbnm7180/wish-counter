//! Модульные тесты для `json_file.rs`.
//!
//! Подключаются в `json_file.rs` строками `#[cfg(test)] #[path = "json_file.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use std::collections::BTreeMap;

use super::*;

/// Пример значения для записи.
fn sample() -> BTreeMap<String, u32> {
    BTreeMap::from([("a".to_owned(), 1), ("b".to_owned(), 2)])
}

#[test]
fn missing_file_reads_as_none() {
    let temp = tempfile::tempdir().unwrap();
    let value: Option<BTreeMap<String, u32>> = read_json(&temp.path().join("none.json")).unwrap();
    assert!(value.is_none());
}

#[test]
fn written_value_is_read_back() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("data.json");

    write_json_atomically(&path, &sample()).unwrap();

    assert_eq!(
        read_json::<BTreeMap<String, u32>>(&path).unwrap(),
        Some(sample())
    );
}

/// После записи временный файл не остаётся.
#[test]
fn temporary_file_is_removed() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("data.json");

    write_json_atomically(&path, &sample()).unwrap();

    let names: Vec<_> = fs::read_dir(temp.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, ["data.json"]);
}

#[test]
fn invalid_json_is_reported_as_corrupted() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("data.json");
    fs::write(&path, "{ broken").unwrap();

    let result = read_json::<BTreeMap<String, u32>>(&path);

    assert!(matches!(result, Err(RepositoryError::Corrupted(_))));
}
