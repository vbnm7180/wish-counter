//! # Модуль игры Genshin Impact
//!
//! Описывает Genshin Impact для приложения. Сейчас модуль только
//! сообщает идентификатор игры. На следующих этапах сюда добавятся:
//! - типы баннеров (персонажи, оружие, стандартный, хроники, новичковый);
//! - правила гаранта (90/80 молитв, мягкий гарант, 50/50, Capturing Radiance);
//! - разбор истории молитв из игры и из файлов Paimon.moe / UIGF.

use domain::game::{GameId, GameInfo, GameModule};

/// Идентификатор Genshin Impact внутри приложения.
pub const GENSHIN_GAME_ID: &str = "genshin";

/// Порядок Genshin Impact в меню игр (первая игра).
const GENSHIN_DISPLAY_ORDER: u32 = 10;

/// Игровой модуль Genshin Impact.
///
/// Структура без полей (unit struct): данных у модуля пока нет,
/// но он реализует трейт [`GameModule`].
#[derive(Debug, Default, Clone, Copy)]
pub struct GenshinModule;

impl GameModule for GenshinModule {
    /// Возвращает описание Genshin Impact.
    fn info(&self) -> GameInfo {
        GameInfo {
            id: GameId::new(GENSHIN_GAME_ID),
            display_order: GENSHIN_DISPLAY_ORDER,
        }
    }
}

// Тесты лежат рядом, в файле `lib.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "lib.tests.rs"]
mod tests;
