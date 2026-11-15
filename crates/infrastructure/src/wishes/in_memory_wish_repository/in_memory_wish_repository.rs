//! Хранилище молитв в оперативной памяти.

use std::collections::HashMap;

use application::ports::{RepositoryError, WishRepository};
use domain::game::GameId;
use domain::wish::Wish;

/// Держит историю молитв в памяти, ничего не сохраняет на диск.
///
/// **Временное решение** до этапа 3: тогда в `bootstrap.rs` его заменит
/// хранилище на SQLite. Остальной код не изменится, потому что работает
/// через трейт `WishRepository`. Хранилище пригодится и в тестах.
#[derive(Debug, Default)]
pub struct InMemoryWishRepository {
    wishes_by_game: HashMap<GameId, Vec<Wish>>,
}

impl InMemoryWishRepository {
    /// Создаёт пустое хранилище.
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавляет молитвы игры и возвращает хранилище (для цепочки вызовов).
    /// Молитвы сортируются по времени, как требует контракт `WishRepository`.
    pub fn with_wishes(mut self, game_id: GameId, wishes: Vec<Wish>) -> Self {
        let stored = self.wishes_by_game.entry(game_id).or_default();
        stored.extend(wishes);
        // Сортировка стабильная: молитвы с одинаковым временем
        // (10 молитв за раз) сохраняют исходный порядок.
        stored.sort_by(|left, right| left.time.cmp(&right.time));
        self
    }
}

impl WishRepository for InMemoryWishRepository {
    /// Возвращает копию молитв игры (пустой список, если их нет).
    fn list_wishes(&self, game_id: &GameId) -> Result<Vec<Wish>, RepositoryError> {
        Ok(self
            .wishes_by_game
            .get(game_id)
            .cloned()
            .unwrap_or_default())
    }
}

// Тесты лежат рядом, в файле `in_memory_wish_repository.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "in_memory_wish_repository.tests.rs"]
mod tests;
