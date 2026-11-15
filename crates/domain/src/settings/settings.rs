//! Пользовательские настройки приложения: тема оформления и язык.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Максимальная длина идентификатора темы.
const MAX_THEME_ID_LENGTH: usize = 32;

/// Тема, которая используется, если пользователь ещё ничего не выбирал.
const DEFAULT_THEME_ID: &str = "dark";

/// Ошибка проверки идентификатора темы.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ThemeIdError {
    /// Пустая строка.
    #[error("идентификатор темы не может быть пустым")]
    Empty,
    /// Строка длиннее `MAX_THEME_ID_LENGTH` (32 символа).
    #[error("идентификатор темы длиннее {MAX_THEME_ID_LENGTH} символов")]
    TooLong,
    /// Есть символы кроме латинских строчных букв, цифр и дефиса.
    #[error("идентификатор темы может содержать только a-z, 0-9 и '-'")]
    InvalidCharacters,
}

/// Идентификатор темы оформления, например `"dark"` или `"light"`.
///
/// Тема хранится как строка, а не как перечисление (`enum`): так новую тему
/// можно добавить только в интерфейсе (CSS-файл + запись в реестре тем),
/// не меняя Rust-код. Rust лишь проверяет, что строка безопасна.
///
/// `try_from = "String"` заставляет serde при чтении из JSON вызвать
/// [`ThemeId::parse`], поэтому некорректное значение не попадёт в программу.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ThemeId(String);

impl ThemeId {
    /// Проверяет строку и создаёт идентификатор темы.
    ///
    /// Допустимы только `a-z`, `0-9` и `-`, длина от 1 до 32 символов.
    /// Ограничение защищает от «мусора» в файле настроек, который мог бы
    /// попасть в атрибут HTML.
    pub fn parse(value: impl Into<String>) -> Result<Self, ThemeIdError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ThemeIdError::Empty);
        }
        if value.len() > MAX_THEME_ID_LENGTH {
            return Err(ThemeIdError::TooLong);
        }
        if !value.chars().all(is_allowed_theme_char) {
            return Err(ThemeIdError::InvalidCharacters);
        }
        Ok(Self(value))
    }

    /// Возвращает идентификатор как строковый срез.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Проверяет, что символ допустим в идентификаторе темы.
fn is_allowed_theme_char(character: char) -> bool {
    character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
}

impl Default for ThemeId {
    /// Тема по умолчанию — тёмная.
    fn default() -> Self {
        Self(DEFAULT_THEME_ID.to_owned())
    }
}

impl TryFrom<String> for ThemeId {
    type Error = ThemeIdError;

    /// Преобразование из `String` с проверкой (используется serde).
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl From<ThemeId> for String {
    /// Обратное преобразование в `String` (используется serde).
    fn from(theme: ThemeId) -> Self {
        theme.0
    }
}

/// Язык интерфейса.
///
/// `rename_all = "lowercase"` + явные `rename` дают в JSON коды
/// `"ru"` и `"en"` — те же, что использует интерфейс.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    /// Русский язык (по умолчанию).
    #[default]
    #[serde(rename = "ru")]
    Russian,
    /// Английский язык.
    #[serde(rename = "en")]
    English,
}

/// Все пользовательские настройки приложения.
///
/// `#[serde(default)]` означает: если в сохранённом файле нет какого-то поля
/// (например, его добавили в новой версии), берётся значение по умолчанию.
/// Так старые файлы настроек остаются совместимыми.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    /// Выбранная тема оформления.
    pub theme: ThemeId,
    /// Выбранный язык интерфейса.
    pub language: Language,
}

// Тесты лежат рядом, в файле `settings.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "settings.tests.rs"]
mod tests;
