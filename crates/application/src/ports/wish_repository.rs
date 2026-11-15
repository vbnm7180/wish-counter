//! Интерфейс хранилища истории молитв.

use domain::game::GameId;
use domain::wish::Wish;

use super::RepositoryError;

/// Хранилище истории молитв.
///
/// Реализация на SQLite появится на этапе 3; до этого используется
/// хранилище в памяти (`infrastructure::wishes::InMemoryWishRepository`).
pub trait WishRepository: Send + Sync {
    /// Возвращает все молитвы игры **в хронологическом порядке**
    /// (от старых к новым) — этого требует расчёт гаранта.
    fn list_wishes(&self, game_id: &GameId) -> Result<Vec<Wish>, RepositoryError>;
}
