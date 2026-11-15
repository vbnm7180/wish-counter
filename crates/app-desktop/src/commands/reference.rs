//! Команды справочных данных.

use application::services::ReferenceSummary;
use domain::game::GameId;
use tauri::State;

use crate::error::CommandError;
use crate::state::AppState;

/// Возвращает сводку о справочнике игры: число персонажей, оружия, баннеров,
/// последний баннер и версию данных.
///
/// Вызов из интерфейса: `invoke('get_reference_summary', { gameId: 'genshin' })`.
#[tauri::command]
pub fn get_reference_summary(
    state: State<'_, AppState>,
    game_id: GameId,
) -> Result<ReferenceSummary, CommandError> {
    Ok(state.reference_service.summary(&game_id)?)
}
