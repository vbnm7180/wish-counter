//! Каталог поддерживаемых игр.

use std::sync::Arc;

use domain::game::{GameId, GameInfo, GameModule};

/// Реестр всех игровых модулей, подключённых к приложению.
///
/// Модули регистрируются один раз при запуске (в `app-desktop`).
/// Остальной код получает список игр отсюда и не знает,
/// какие именно игры существуют.
///
/// `Arc<dyn GameModule>` — это «умный указатель» с подсчётом ссылок
/// на любой объект, реализующий [`GameModule`]. Он позволяет хранить
/// модули разных типов в одном векторе и безопасно делить их между потоками.
#[derive(Default)]
pub struct GameCatalog {
    modules: Vec<Arc<dyn GameModule>>,
}

impl GameCatalog {
    /// Создаёт пустой каталог.
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавляет игровой модуль в каталог.
    ///
    /// Метод принимает `self` по значению и возвращает его, поэтому
    /// вызовы можно писать цепочкой: `GameCatalog::new().with_module(a).with_module(b)`.
    pub fn with_module(mut self, module: Arc<dyn GameModule>) -> Self {
        self.modules.push(module);
        self
    }

    /// Возвращает описания всех игр, отсортированные по `display_order`.
    pub fn list_games(&self) -> Vec<GameInfo> {
        let mut games: Vec<GameInfo> = self.modules.iter().map(|module| module.info()).collect();
        games.sort_by_key(|game| game.display_order);
        games
    }

    /// Перебирает все зарегистрированные модули игр.
    ///
    /// `impl Iterator` — «какой-то итератор»: конкретный тип скрыт,
    /// вызывающему коду важно только, что по нему можно пройти циклом.
    pub fn modules(&self) -> impl Iterator<Item = &Arc<dyn GameModule>> {
        self.modules.iter()
    }

    /// Ищет модуль игры по идентификатору.
    ///
    /// Возвращает `None`, если игра не зарегистрирована.
    pub fn find(&self, id: &GameId) -> Option<Arc<dyn GameModule>> {
        self.modules
            .iter()
            .find(|module| &module.info().id == id)
            .cloned()
    }
}

// Тесты лежат рядом, в файле `game_catalog.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "game_catalog.tests.rs"]
mod tests;
