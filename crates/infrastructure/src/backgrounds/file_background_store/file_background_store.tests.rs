//! Модульные тесты для `file_background_store.rs`.
//!
//! Подключаются в `file_background_store.rs` строками
//! `#[cfg(test)] #[path = "file_background_store.tests.rs"] mod tests;`.

use domain::background::ImageFormat;

use super::*;

/// Хранилище во временной папке (папка `backgrounds` внутри ещё не создана).
fn store() -> (tempfile::TempDir, FileBackgroundStore) {
    let temp = tempfile::tempdir().unwrap();
    let store = FileBackgroundStore::new(temp.path().join("backgrounds"));
    (temp, store)
}

/// Картинка с идентификатором `id` в формате `format`.
fn image(id: &str, format: ImageFormat) -> BackgroundImage {
    BackgroundImage {
        id: BackgroundId::parse(id).unwrap(),
        format,
    }
}

#[test]
fn missing_folder_is_empty_list() {
    let (_temp, store) = store();

    assert!(store.list().unwrap().is_empty());
    assert_eq!(
        store.load(&BackgroundId::parse("abc").unwrap()).unwrap(),
        None
    );
}

#[test]
fn saved_image_is_listed_and_loaded() {
    let (temp, store) = store();
    let webp = image("abc", ImageFormat::Webp);

    store.save(&webp, &[1, 2, 3]).unwrap();

    assert!(temp.path().join("backgrounds").join("abc.webp").is_file());
    assert_eq!(store.list().unwrap(), vec![webp.clone()]);
    assert_eq!(store.load(&webp.id).unwrap(), Some((webp, vec![1, 2, 3])));
}

#[test]
fn images_are_listed_in_order_of_adding() {
    let (_temp, store) = store();
    let first = image("bbb", ImageFormat::Png);
    let second = image("aaa", ImageFormat::Jpeg);

    store.save(&first, &[1]).unwrap();
    store.save(&second, &[2]).unwrap();

    assert_eq!(store.list().unwrap(), vec![first, second]);
}

#[test]
fn saving_existing_image_keeps_first_file() {
    let (_temp, store) = store();
    let png = image("abc", ImageFormat::Png);

    store.save(&png, &[1]).unwrap();
    store.save(&png, &[2]).unwrap();

    assert_eq!(store.load(&png.id).unwrap().unwrap().1, vec![1]);
    assert_eq!(store.list().unwrap().len(), 1);
}

#[test]
fn removed_image_disappears_and_missing_is_not_error() {
    let (temp, store) = store();
    let jpeg = image("abc", ImageFormat::Jpeg);
    store.save(&jpeg, &[1]).unwrap();

    store.remove(&jpeg.id).unwrap();
    store.remove(&jpeg.id).unwrap();

    assert!(store.list().unwrap().is_empty());
    assert!(!temp.path().join("backgrounds").join("abc.jpg").exists());
}

/// Файл картинки удалён вручную — картинка из списка не читается, но и ошибки нет.
#[test]
fn image_with_missing_file_loads_as_none() {
    let (temp, store) = store();
    let png = image("abc", ImageFormat::Png);
    store.save(&png, &[1]).unwrap();
    fs::remove_file(temp.path().join("backgrounds").join("abc.png")).unwrap();

    assert_eq!(store.load(&png.id).unwrap(), None);
}

/// Испорченный список — ошибка «повреждённые данные», а не паника.
#[test]
fn corrupted_index_is_reported() {
    let (temp, store) = store();
    let folder = temp.path().join("backgrounds");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join(INDEX_FILE_NAME), "не json").unwrap();

    assert!(matches!(store.list(), Err(RepositoryError::Corrupted(_))));
}
