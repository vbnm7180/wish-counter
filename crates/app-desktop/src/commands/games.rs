//! Команды, связанные с играми: список игр и статистика молитв.

use application::services::GameStatistics;
use domain::game::{GameId, GameInfo};
use tauri::State;

use crate::error::CommandError;
use crate::state::AppState;

/// Возвращает список поддерживаемых игр.
///
/// Вызов из интерфейса: `invoke('list_games')`.
#[tauri::command]
pub fn list_games(state: State<'_, AppState>) -> Vec<GameInfo> {
    state.game_catalog.list_games()
}

/// Возвращает статистику молитв игры: итоги и карточки баннеров.
///
/// Вызов из интерфейса: `invoke('get_game_statistics', { gameId: 'genshin' })`.
/// Tauri переводит имя параметра `game_id` в `gameId` для JavaScript.
#[tauri::command]
pub fn get_game_statistics(
    state: State<'_, AppState>,
    game_id: GameId,
) -> Result<GameStatistics, CommandError> {
    Ok(state.statistics_service.game_statistics(&game_id)?)
}
