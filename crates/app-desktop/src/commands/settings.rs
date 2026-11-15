//! Команды чтения и сохранения пользовательских настроек.

use domain::settings::AppSettings;
use tauri::State;

use crate::error::CommandError;
use crate::state::AppState;

/// Возвращает текущие настройки.
///
/// Вызов из интерфейса: `invoke('get_settings')`.
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    Ok(state.settings_service.get_settings()?)
}

/// Сохраняет новые настройки и возвращает их.
///
/// Вызов из интерфейса: `invoke('update_settings', { settings })`.
/// Tauri сам проверит JSON: например, недопустимая тема будет отклонена
/// ещё до вызова этой функции (см. `ThemeId` в домене).
#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, CommandError> {
    Ok(state.settings_service.update_settings(settings)?)
}
