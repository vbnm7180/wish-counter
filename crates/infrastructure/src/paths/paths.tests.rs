//! Модульные тесты для `paths.rs`.
//!
//! Подключаются в `paths.rs` строками `#[cfg(test)] #[path = "paths.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

#[test]
fn paths_are_built_from_data_dir() {
    let paths = AppPaths::from_data_dir("root");

    assert_eq!(paths.data_dir, PathBuf::from("root"));
    assert_eq!(paths.logs_dir, PathBuf::from("root").join("logs"));
    assert_eq!(
        paths.settings_file,
        PathBuf::from("root").join("settings.json")
    );
}

#[test]
fn portable_mode_points_next_to_executable() {
    let paths = AppPaths::resolve(StorageMode::Portable).unwrap();
    let executable_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    assert_eq!(paths.data_dir, executable_dir.join("data"));
}

#[test]
fn ensure_directories_creates_folders() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::from_data_dir(temp.path().join("data"));

    paths.ensure_directories().unwrap();

    assert!(paths.data_dir.is_dir());
    assert!(paths.logs_dir.is_dir());
}
