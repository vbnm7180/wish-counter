//! Команды, связанные со списком игр.

use domain::game::GameInfo;
use tauri::State;

use crate::state::AppState;

/// Возвращает список поддерживаемых игр.
///
/// Вызов из интерфейса: `invoke('list_games')`.
#[tauri::command]
pub fn list_games(state: State<'_, AppState>) -> Vec<GameInfo> {
    state.game_catalog.list_games()
}
