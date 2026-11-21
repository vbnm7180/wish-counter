//! Интерфейс хранилища истории молитв.

use domain::game::GameId;
use domain::wish::Wish;

use super::RepositoryError;

/// Хранилище истории молитв.
///
/// Основная реализация — SQLite (`infrastructure::wishes::SqliteWishRepository`).
pub trait WishRepository: Send + Sync {
    /// Возвращает все молитвы игры **в хронологическом порядке**
    /// (от старых к новым) — этого требует расчёт гаранта.
    ///
    /// Молитвы с одинаковым временем (10 молитв за раз) возвращаются
    /// в том порядке, в котором были добавлены.
    fn list_wishes(&self, game_id: &GameId) -> Result<Vec<Wish>, RepositoryError>;

    /// Добавляет молитвы игры и возвращает, сколько из них новых.
    ///
    /// - Молитвы нужно передавать **в хронологическом порядке**.
    /// - Молитва с уже сохранённым идентификатором пропускается, поэтому
    ///   повторный импорт той же истории безопасен.
    /// - Добавление атомарное: при ошибке не сохраняется ничего.
    fn add_wishes(&self, game_id: &GameId, wishes: &[Wish]) -> Result<usize, RepositoryError>;

    /// Удаляет все молитвы игры. Используется при смене аккаунта.
    fn delete_wishes(&self, game_id: &GameId) -> Result<(), RepositoryError>;
}
