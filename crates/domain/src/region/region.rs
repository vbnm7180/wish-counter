//! Сервер (регион) игрока.
//!
//! Время в истории молитв записано по часовому поясу сервера игрока.
//! Чтобы сопоставить молитву с баннером, нужно знать этот пояс.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Сервер игры. Набор и смещения — как в Paimon.moe (`src/stores/server.js`).
///
/// В JSON — строка в нижнем регистре: `"america"`, `"europe"`, `"asia"`, `"china"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerRegion {
    /// Америка (UTC−5).
    America,
    /// Европа (UTC+1).
    Europe,
    /// Азия, а также TW/HK/MO (UTC+8).
    Asia,
    /// Китайские серверы (UTC+8).
    China,
}

/// Часовой пояс, в котором записано время баннеров в данных Paimon.moe (UTC+8).
pub const ASIA_UTC_OFFSET_HOURS: i64 = 8;

impl ServerRegion {
    /// Все серверы в порядке отображения.
    pub const ALL: [Self; 4] = [Self::America, Self::Europe, Self::Asia, Self::China];

    /// Смещение часового пояса сервера относительно UTC в часах.
    pub fn utc_offset_hours(self) -> i64 {
        match self {
            Self::America => -5,
            Self::Europe => 1,
            Self::Asia | Self::China => ASIA_UTC_OFFSET_HOURS,
        }
    }

    /// На сколько часов время сервера отстаёт от азиатского (UTC+8).
    /// Для Америки — 13, для Европы — 7, для Азии — 0.
    pub fn hours_behind_asia(self) -> i64 {
        ASIA_UTC_OFFSET_HOURS - self.utc_offset_hours()
    }

    /// Идентификатор в нижнем регистре (как в JSON и в базе данных).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::America => "america",
            Self::Europe => "europe",
            Self::Asia => "asia",
            Self::China => "china",
        }
    }

    /// Находит сервер по идентификатору (без учёта регистра:
    /// в файлах Paimon.moe записано `"America"`, у нас — `"america"`).
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|region| region.as_str().eq_ignore_ascii_case(value))
    }
}

impl fmt::Display for ServerRegion {
    /// Печатает идентификатор сервера.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

// Тесты лежат рядом, в файле `region.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "region.tests.rs"]
mod tests;
