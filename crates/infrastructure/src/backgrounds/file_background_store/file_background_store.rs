//! Картинки фона в файлах: `data/backgrounds/<идентификатор>.<png|jpg|webp>`
//! и список картинок `data/backgrounds/backgrounds.json`.
//!
//! Содержимое папки намеренно **не перебирается** (`read_dir` не используется):
//! программа обращается только к файлам с заранее известными именами из списка.
//! Перебор папок вместе с сетевым кодом в одной программе антивирусная эвристика
//! Касперского приняла за поведение «похитителя паролей» (ADR-036).

use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use application::ports::{BackgroundStore, RepositoryError};
use domain::background::{BackgroundId, BackgroundImage};

use crate::files::{read_bytes, read_json, write_bytes_atomically, write_json_atomically};

/// Имя файла со списком картинок.
const INDEX_FILE_NAME: &str = "backgrounds.json";

/// Хранилище картинок фона в папке на диске.
pub struct FileBackgroundStore {
    directory: PathBuf,
}

impl FileBackgroundStore {
    /// Создаёт хранилище в указанной папке (создаётся при первой записи, если её нет).
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    /// Путь к файлу картинки. Идентификатор уже проверен типом [`BackgroundId`]
    /// (только `a-z`, `0-9`, `-`), поэтому выйти за пределы папки через него нельзя.
    fn image_path(&self, image: &BackgroundImage) -> PathBuf {
        self.directory.join(file_name(image))
    }

    /// Путь к списку картинок.
    fn index_path(&self) -> PathBuf {
        self.directory.join(INDEX_FILE_NAME)
    }

    /// Читает список картинок; нет файла — пустой список.
    fn read_index(&self) -> Result<Vec<BackgroundImage>, RepositoryError> {
        Ok(read_json(&self.index_path())?.unwrap_or_default())
    }

    /// Записывает список картинок атомарно.
    fn write_index(&self, images: &[BackgroundImage]) -> Result<(), RepositoryError> {
        write_json_atomically(&self.index_path(), &images)
    }
}

impl BackgroundStore for FileBackgroundStore {
    /// Картинки из списка в порядке добавления.
    fn list(&self) -> Result<Vec<BackgroundImage>, RepositoryError> {
        self.read_index()
    }

    /// Читает картинку; `None`, если её нет в списке или файл пропал.
    fn load(
        &self,
        id: &BackgroundId,
    ) -> Result<Option<(BackgroundImage, Vec<u8>)>, RepositoryError> {
        let Some(image) = self.read_index()?.into_iter().find(|image| &image.id == id) else {
            return Ok(None);
        };
        Ok(read_bytes(&self.image_path(&image))?.map(|bytes| (image, bytes)))
    }

    /// Сохраняет картинку, если её ещё нет: сначала файл, затем запись в списке
    /// (если программа упадёт между шагами, в списке не будет ссылки на несуществующий файл).
    fn save(&self, image: &BackgroundImage, bytes: &[u8]) -> Result<(), RepositoryError> {
        let mut images = self.read_index()?;
        if images.iter().any(|saved| saved.id == image.id) {
            return Ok(());
        }
        fs::create_dir_all(&self.directory)
            .map_err(|error| RepositoryError::Io(error.to_string()))?;
        write_bytes_atomically(&self.image_path(image), bytes)?;
        images.push(image.clone());
        self.write_index(&images)
    }

    /// Удаляет картинку: сначала из списка, затем файл.
    fn remove(&self, id: &BackgroundId) -> Result<(), RepositoryError> {
        let mut images = self.read_index()?;
        let Some(position) = images.iter().position(|image| &image.id == id) else {
            return Ok(());
        };
        let removed = images.remove(position);
        self.write_index(&images)?;
        match fs::remove_file(self.image_path(&removed)) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(RepositoryError::Io(error.to_string())),
        }
    }
}

/// Имя файла картинки: `<идентификатор>.<расширение формата>`.
fn file_name(image: &BackgroundImage) -> String {
    format!("{}.{}", image.id.as_str(), image.format.extension())
}

// Тесты лежат рядом, в файле `file_background_store.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "file_background_store.tests.rs"]
mod tests;
