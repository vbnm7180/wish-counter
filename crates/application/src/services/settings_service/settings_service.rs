//! Сервис пользовательских настроек.

use std::sync::Arc;

use domain::settings::AppSettings;

use crate::ports::{RepositoryError, SettingsRepository};

/// Читает и сохраняет настройки приложения.
///
/// Сервис не знает, где именно лежат настройки: он получает хранилище
/// через конструктор (внедрение зависимости, Dependency Injection).
pub struct SettingsService {
    repository: Arc<dyn SettingsRepository>,
}

impl SettingsService {
    /// Создаёт сервис поверх переданного хранилища.
    pub fn new(repository: Arc<dyn SettingsRepository>) -> Self {
        Self { repository }
    }

    /// Возвращает текущие настройки.
    ///
    /// - Настройки ещё не сохранялись → значения по умолчанию.
    /// - Файл повреждён → значения по умолчанию и предупреждение в лог
    ///   (приложение должно запуститься, даже если файл испорчен).
    /// - Ошибка диска → ошибка передаётся вызывающему коду.
    pub fn get_settings(&self) -> Result<AppSettings, RepositoryError> {
        match self.repository.load() {
            Ok(Some(settings)) => Ok(settings),
            Ok(None) => Ok(AppSettings::default()),
            Err(RepositoryError::Corrupted(reason)) => {
                tracing::warn!(%reason, "файл настроек повреждён, используются значения по умолчанию");
                Ok(AppSettings::default())
            }
            Err(error) => Err(error),
        }
    }

    /// Сохраняет новые настройки и возвращает их.
    ///
    /// Проверка значений (например, корректность темы) уже выполнена
    /// типами домена при создании [`AppSettings`].
    pub fn update_settings(&self, settings: AppSettings) -> Result<AppSettings, RepositoryError> {
        self.repository.save(&settings)?;
        tracing::info!(theme = settings.theme.as_str(), language = ?settings.language, "настройки сохранены");
        Ok(settings)
    }
}

// Тесты лежат рядом, в файле `settings_service.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "settings_service.tests.rs"]
mod tests;
