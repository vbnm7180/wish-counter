//! Фон приложения: картинки, которые пользователь добавил сам.
//!
//! Картинка копируется в папку данных приложения (в portable-режиме — `data/backgrounds/`)
//! и получает идентификатор, вычисленный по её содержимому. Поэтому один и тот же файл,
//! добавленный дважды, хранится один раз, а идентификатор безопасно использовать
//! как имя файла и как часть адреса картинки в интерфейсе.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Максимальная длина идентификатора фона.
const MAX_BACKGROUND_ID_LENGTH: usize = 64;

/// Начальное значение хэша FNV-1a (64 бита) — константа из описания алгоритма.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// Множитель хэша FNV-1a (64 бита) — константа из описания алгоритма.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Первые 8 байт любого PNG-файла.
const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
/// Первые 3 байта любого JPEG-файла.
const JPEG_SIGNATURE: [u8; 3] = [0xFF, 0xD8, 0xFF];
/// WebP: файл-контейнер RIFF (байты 0..4) с типом `WEBP` (байты 8..12).
const RIFF_SIGNATURE: &[u8; 4] = b"RIFF";
/// Тип содержимого RIFF для WebP.
const WEBP_FOURCC: &[u8; 4] = b"WEBP";
/// С какого байта в RIFF записан тип содержимого.
const RIFF_FOURCC_OFFSET: usize = 8;

/// Ошибка проверки идентификатора фона.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BackgroundIdError {
    /// Пустая строка.
    #[error("идентификатор фона не может быть пустым")]
    Empty,
    /// Строка длиннее `MAX_BACKGROUND_ID_LENGTH` (64 символа).
    #[error("идентификатор фона длиннее {MAX_BACKGROUND_ID_LENGTH} символов")]
    TooLong,
    /// Есть символы кроме латинских строчных букв, цифр и дефиса.
    #[error("идентификатор фона может содержать только a-z, 0-9 и '-'")]
    InvalidCharacters,
}

/// Идентификатор фона, например `"3f9a0c1d2e4b5a69"`.
///
/// Допустимы только `a-z`, `0-9` и `-`: идентификатор становится именем файла
/// и частью адреса `wcimg://`, поэтому через него нельзя выйти за пределы папки.
/// `try_from = "String"` заставляет serde проверять строку при чтении настроек.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct BackgroundId(String);

impl BackgroundId {
    /// Проверяет строку и создаёт идентификатор.
    pub fn parse(value: impl Into<String>) -> Result<Self, BackgroundIdError> {
        let value = value.into();
        if value.is_empty() {
            return Err(BackgroundIdError::Empty);
        }
        if value.len() > MAX_BACKGROUND_ID_LENGTH {
            return Err(BackgroundIdError::TooLong);
        }
        if !value.chars().all(is_allowed_id_char) {
            return Err(BackgroundIdError::InvalidCharacters);
        }
        Ok(Self(value))
    }

    /// Идентификатор по содержимому картинки: 16 шестнадцатеричных цифр хэша FNV-1a.
    ///
    /// FNV-1a — простой некриптографический хэш: здесь он нужен только для того,
    /// чтобы одинаковые файлы получали одинаковое имя (защита от подделки не требуется).
    pub fn from_content(bytes: &[u8]) -> Self {
        Self(format!("{:016x}", fnv1a_64(bytes)))
    }

    /// Возвращает идентификатор как строковый срез.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for BackgroundId {
    type Error = BackgroundIdError;

    /// Преобразование из `String` с проверкой (используется serde).
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl From<BackgroundId> for String {
    /// Обратное преобразование в `String` (используется serde).
    fn from(id: BackgroundId) -> Self {
        id.0
    }
}

impl std::fmt::Display for BackgroundId {
    /// Выводит идентификатор как есть (для логов).
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Проверяет, что символ допустим в идентификаторе фона.
fn is_allowed_id_char(character: char) -> bool {
    character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
}

/// Хэш FNV-1a (64 бита): для каждого байта — «исключающее ИЛИ», затем умножение.
///
/// `wrapping_mul` — умножение с переполнением «по кругу» (по модулю 2⁶⁴):
/// так задумано алгоритмом, а обычное `*` в отладочной сборке остановило бы программу.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV_OFFSET_BASIS, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
    })
}

/// Формат картинки фона. Определяется по первым байтам файла, а не по расширению:
/// расширение легко переименовать, а содержимое — нет.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    /// PNG.
    Png,
    /// JPEG.
    Jpeg,
    /// WebP.
    Webp,
}

impl ImageFormat {
    /// Все поддерживаемые форматы.
    pub const ALL: [ImageFormat; 3] = [ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::Webp];

    /// Определяет формат по содержимому; `None` — формат не поддерживается.
    pub fn detect(bytes: &[u8]) -> Option<Self> {
        if bytes.starts_with(&PNG_SIGNATURE) {
            Some(Self::Png)
        } else if bytes.starts_with(&JPEG_SIGNATURE) {
            Some(Self::Jpeg)
        } else if is_webp(bytes) {
            Some(Self::Webp)
        } else {
            None
        }
    }

    /// Расширение файла без точки (`"png"`, `"jpg"`, `"webp"`).
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
        }
    }

    /// Формат по расширению файла (без учёта регистра); `None` — неизвестное расширение.
    pub fn from_extension(extension: &str) -> Option<Self> {
        let extension = extension.to_ascii_lowercase();
        match extension.as_str() {
            "jpeg" => Some(Self::Jpeg),
            other => Self::ALL
                .into_iter()
                .find(|format| format.extension() == other),
        }
    }

    /// MIME-тип для HTTP-ответа (`"image/png"` и т.д.).
    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
    }
}

/// Проверяет, что байты — WebP: контейнер RIFF с типом `WEBP`.
fn is_webp(bytes: &[u8]) -> bool {
    bytes.starts_with(RIFF_SIGNATURE)
        && bytes.get(RIFF_FOURCC_OFFSET..RIFF_FOURCC_OFFSET + WEBP_FOURCC.len())
            == Some(WEBP_FOURCC.as_slice())
}

/// Сохранённая картинка фона: идентификатор и формат.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackgroundImage {
    /// Идентификатор (имя файла без расширения).
    pub id: BackgroundId,
    /// Формат картинки.
    pub format: ImageFormat,
}

// Тесты лежат рядом, в файле `background.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "background.tests.rs"]
mod tests;
