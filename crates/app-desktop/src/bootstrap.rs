//! Сборка приложения (composition root).
//!
//! Это единственное место, где создаются конкретные реализации
//! (файловое хранилище, игровые модули) и передаются в сервисы.
//! Остальной код зависит только от абстракций (трейтов).

use std::sync::Arc;

use application::services::{GameCatalog, SettingsService};
use game_genshin::GenshinModule;
use infrastructure::logging::init_logging;
use infrastructure::paths::{AppPaths, StorageMode};
use infrastructure::settings::JsonSettingsRepository;

use crate::commands;
use crate::error::StartupError;
use crate::state::AppState;

/// Режим хранения данных. Для будущей версии с установщиком
/// достаточно поменять его на [`StorageMode::UserProfile`].
const STORAGE_MODE: StorageMode = StorageMode::Portable;

/// Подготавливает окружение и запускает окно приложения.
///
/// Функция блокируется, пока пользователь не закроет окно.
pub fn run() -> Result<(), StartupError> {
    let paths = AppPaths::resolve(STORAGE_MODE)?;
    paths
        .ensure_directories()
        .map_err(StartupError::DataDirectory)?;
    // Переменная `_log_guard` должна жить до конца `run`, иначе логи оборвутся.
    let _log_guard = init_logging(&paths.logs_dir)?;
    tracing::info!(data_dir = %paths.data_dir.display(), "запуск Wish Counter");

    let state = build_state(&paths);

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::games::list_games,
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())?;

    tracing::info!("Wish Counter завершил работу");
    Ok(())
}

/// Создаёт сервисы приложения и связывает их с реализациями.
fn build_state(paths: &AppPaths) -> AppState {
    let settings_repository = Arc::new(JsonSettingsRepository::new(&paths.settings_file));
    let game_catalog = GameCatalog::new().with_module(Arc::new(GenshinModule));

    AppState {
        game_catalog,
        settings_service: SettingsService::new(settings_repository),
    }
}
