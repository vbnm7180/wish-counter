//! Интерфейс хранилища картинок фона, добавленных пользователем.

use domain::background::{BackgroundId, BackgroundImage};

use super::RepositoryError;

/// Хранилище картинок фона (в portable-режиме — `data/backgrounds/`).
pub trait BackgroundStore: Send + Sync {
    /// Все сохранённые картинки в порядке добавления (старые первыми).
    fn list(&self) -> Result<Vec<BackgroundImage>, RepositoryError>;

    /// Читает картинку; `None`, если её нет.
    fn load(
        &self,
        id: &BackgroundId,
    ) -> Result<Option<(BackgroundImage, Vec<u8>)>, RepositoryError>;

    /// Сохраняет картинку. Если картинка с таким идентификатором уже есть, ничего не меняет.
    fn save(&self, image: &BackgroundImage, bytes: &[u8]) -> Result<(), RepositoryError>;

    /// Удаляет картинку. Если её нет, это не ошибка.
    fn remove(&self, id: &BackgroundId) -> Result<(), RepositoryError>;
}
