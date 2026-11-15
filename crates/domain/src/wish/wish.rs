//! Молитва (одна крутка) и её составные части.
//!
//! Эти типы общие для всех игр: у любой гача-игры есть баннер, предмет,
//! его редкость и время получения. Особенности конкретной игры
//! (коды баннеров, правила гаранта) описываются в её модуле (`crates/games/*`).

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Уникальный идентификатор молитвы, выданный игрой.
///
/// Хранится строкой, а не числом: игра выдаёт 19-значные числа, а тип
/// `number` в JavaScript точно хранит только до 16 цифр. Число исказилось бы
/// при передаче в интерфейс.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WishId(String);

impl WishId {
    /// Создаёт идентификатор молитвы.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Возвращает идентификатор как строковый срез.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Идентификатор предмета (персонажа или оружия), например `"hu_tao"`.
///
/// Формат совпадает с идентификаторами Paimon.moe, чтобы импорт
/// их файлов не требовал перевода названий (этапы 4 и 6).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemId(String);

impl ItemId {
    /// Создаёт идентификатор предмета.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Возвращает идентификатор как строковый срез.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ItemId {
    /// Печатает идентификатор без обёрток.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Код баннера в игре (в API HoYoverse — `gacha_type`), например `301`.
///
/// Коды конкретной игры объявлены в её модуле
/// (для Genshin — `game_genshin::banner_codes`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BannerCode(u32);

impl BannerCode {
    /// Создаёт код баннера.
    ///
    /// `const fn` можно вызывать при компиляции, поэтому коды баннеров
    /// объявляются как константы: `const CHARACTER: BannerCode = BannerCode::new(301);`.
    pub const fn new(code: u32) -> Self {
        Self(code)
    }

    /// Возвращает числовое значение кода.
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// Тип выпавшего предмета.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    /// Персонаж.
    Character,
    /// Оружие (в HSR — световой конус, в ZZZ — амплификатор).
    Weapon,
}

/// Ошибка создания редкости из числа.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RarityError {
    /// Число не равно 3, 4 или 5.
    #[error("недопустимая редкость {0}: ожидается 3, 4 или 5")]
    Unsupported(u8),
}

/// Редкость предмета в звёздах.
///
/// В JSON хранится числом (`3`, `4`, `5`), как в игре и в Paimon.moe.
/// Для этого serde преобразует значение через `u8` (`into` / `try_from`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
pub enum Rarity {
    /// 3★ — обычное оружие.
    Three,
    /// 4★ — редкий предмет.
    Four,
    /// 5★ — легендарный предмет.
    Five,
}

impl TryFrom<u8> for Rarity {
    type Error = RarityError;

    /// Превращает число звёзд в редкость.
    fn try_from(stars: u8) -> Result<Self, Self::Error> {
        match stars {
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            other => Err(RarityError::Unsupported(other)),
        }
    }
}

impl From<Rarity> for u8 {
    /// Превращает редкость в число звёзд.
    fn from(rarity: Rarity) -> Self {
        match rarity {
            Rarity::Three => 3,
            Rarity::Four => 4,
            Rarity::Five => 5,
        }
    }
}

/// Шаблон времени молитвы: `0` — любая цифра, остальные символы — как есть.
const WISH_TIME_PATTERN: &str = "0000-00-00 00:00:00";

/// Ошибка проверки времени молитвы.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WishTimeError {
    /// Строка не соответствует формату `ГГГГ-ММ-ДД чч:мм:сс`.
    #[error("время молитвы «{0}» не соответствует формату ГГГГ-ММ-ДД чч:мм:сс")]
    InvalidFormat(String),
}

/// Время молитвы по серверу игры в формате `ГГГГ-ММ-ДД чч:мм:сс`.
///
/// Это тот же формат, что в истории молитв игры и в Paimon.moe.
/// Строки такого вида сортируются по алфавиту в том же порядке, что и по
/// времени, поэтому для сравнения не нужна библиотека дат.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct WishTime(String);

impl WishTime {
    /// Проверяет строку и создаёт время молитвы.
    pub fn parse(value: impl Into<String>) -> Result<Self, WishTimeError> {
        let value = value.into();
        if matches_wish_time_pattern(&value) {
            Ok(Self(value))
        } else {
            Err(WishTimeError::InvalidFormat(value))
        }
    }

    /// Возвращает время как строковый срез.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Проверяет, что строка совпадает с [`WISH_TIME_PATTERN`] посимвольно.
fn matches_wish_time_pattern(value: &str) -> bool {
    value.len() == WISH_TIME_PATTERN.len()
        && value
            .bytes()
            .zip(WISH_TIME_PATTERN.bytes())
            .all(|(actual, expected)| match expected {
                b'0' => actual.is_ascii_digit(),
                _ => actual == expected,
            })
}

impl TryFrom<String> for WishTime {
    type Error = WishTimeError;

    /// Преобразование из `String` с проверкой (используется serde).
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl From<WishTime> for String {
    /// Обратное преобразование в `String` (используется serde).
    fn from(time: WishTime) -> Self {
        time.0
    }
}

/// Одна молитва из истории игрока.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wish {
    /// Идентификатор, выданный игрой.
    pub id: WishId,
    /// Код баннера, на котором сделана молитва.
    pub banner_code: BannerCode,
    /// Выпавший предмет.
    pub item_id: ItemId,
    /// Тип предмета: персонаж или оружие.
    pub item_kind: ItemKind,
    /// Редкость предмета.
    pub rarity: Rarity,
    /// Время молитвы по серверу игры.
    pub time: WishTime,
}

// Тесты лежат рядом, в файле `wish.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "wish.tests.rs"]
mod tests;
