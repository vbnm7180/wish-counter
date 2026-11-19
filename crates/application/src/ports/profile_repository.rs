//! Интерфейс хранилища профилей игр (сервер и UID игрока в конкретной игре).

use domain::game::GameId;
use domain::profile::PlayerUid;
use domain::region::ServerRegion;

use super::RepositoryError;

/// Хранилище профилей игр. Профиль хранится вместе с историей молитв
/// (в базе), потому что относится к аккаунту, а не к приложению.
pub trait ProfileRepository: Send + Sync {
    /// Возвращает сервер игрока; `None`, если ещё не выбран.
    fn server_region(&self, game_id: &GameId) -> Result<Option<ServerRegion>, RepositoryError>;

    /// Сохраняет сервер игрока (заменяя прежний).
    fn set_server_region(
        &self,
        game_id: &GameId,
        region: ServerRegion,
    ) -> Result<(), RepositoryError>;

    /// Возвращает UID игрока; `None`, если неизвестен.
    fn uid(&self, game_id: &GameId) -> Result<Option<PlayerUid>, RepositoryError>;

    /// Сохраняет UID игрока (заменяя прежний).
    fn set_uid(&self, game_id: &GameId, uid: &PlayerUid) -> Result<(), RepositoryError>;
}
