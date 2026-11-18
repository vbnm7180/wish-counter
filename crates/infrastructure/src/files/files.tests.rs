//! Модульные тесты для `files.rs`.
//!
//! Подключаются в `files.rs` строками `#[cfg(test)] #[path = "files.tests.rs"] mod tests;`.
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

#[test]
fn bytes_are_written_and_read_back() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("image.png");

    write_bytes_atomically(&path, &[1, 2, 3]).unwrap();

    assert_eq!(read_bytes(&path).unwrap(), Some(vec![1, 2, 3]));
    assert_eq!(read_bytes(&temp.path().join("none.png")).unwrap(), None);
}

#[test]
fn safe_file_stems() {
    for safe in ["genshin", "hu_tao", "primordial_jade_winged-spear", "a1"] {
        assert!(is_safe_file_stem(safe), "{safe}");
    }
    for unsafe_name in ["", "../x", "a/b", r"a\b", "C:", "Hu_Tao", "a.png"] {
        assert!(!is_safe_file_stem(unsafe_name), "{unsafe_name}");
    }
}
