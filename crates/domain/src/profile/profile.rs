//! Профиль игрока в конкретной игре: сервер и UID.
//!
//! Профиль относится к аккаунту, а не к приложению, поэтому хранится вместе
//! с историей молитв (в базе), а не в настройках.

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::region::ServerRegion;

/// Наименьшая допустимая длина UID (в цифрах).
const MIN_UID_LENGTH: usize = 6;
/// Наибольшая допустимая длина UID (в цифрах). В Genshin сейчас 9–10 цифр.
const MAX_UID_LENGTH: usize = 12;

/// Ошибка проверки UID.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlayerUidError {
    /// В UID есть что-то кроме цифр.
    #[error("UID должен состоять только из цифр")]
    NotDigits,
    /// UID слишком короткий или слишком длинный.
    #[error("UID должен содержать от {MIN_UID_LENGTH} до {MAX_UID_LENGTH} цифр")]
    InvalidLength,
}

/// UID игрока — публичный номер аккаунта в игре (например, `700000000`).
///
/// Не секрет (его видно в игре и в профиле), но и не нужен никому, кроме
/// самого игрока: из приложения никуда не отправляется. Нужен для экспорта
/// истории в форматы, где он обязателен.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PlayerUid(String);

impl PlayerUid {
    /// Проверяет строку (пробелы по краям убираются) и создаёт UID.
    pub fn parse(value: &str) -> Result<Self, PlayerUidError> {
        let value = value.trim();
        if !value.chars().all(|c| c.is_ascii_digit()) {
            return Err(PlayerUidError::NotDigits);
        }
        if !(MIN_UID_LENGTH..=MAX_UID_LENGTH).contains(&value.len()) {
            return Err(PlayerUidError::InvalidLength);
        }
        Ok(Self(value.to_owned()))
    }

    /// Возвращает UID как строковый срез.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PlayerUid {
    /// Печатает UID.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl TryFrom<String> for PlayerUid {
    type Error = PlayerUidError;

    /// Преобразование из `String` с проверкой (используется serde).
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<PlayerUid> for String {
    /// Обратное преобразование в `String` (используется serde).
    fn from(uid: PlayerUid) -> Self {
        uid.0
    }
}

/// Профиль игрока в игре. Любое поле может быть ещё не заполнено.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameProfile {
    /// Сервер игрока (нужен для часового пояса истории).
    pub server_region: Option<ServerRegion>,
    /// UID игрока (нужен для экспорта).
    pub uid: Option<PlayerUid>,
}

// Тесты лежат рядом, в файле `profile.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "profile.tests.rs"]
mod tests;
