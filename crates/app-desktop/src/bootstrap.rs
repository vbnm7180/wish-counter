//! Сборка приложения (composition root).
//!
//! Это единственное место, где создаются конкретные реализации
//! (файловое хранилище, игровые модули) и передаются в сервисы.
//! Остальной код зависит только от абстракций (трейтов).

use std::sync::Arc;

use application::services::{GameCatalog, ReferenceService, SettingsService, StatisticsService};
use game_genshin::GenshinModule;
use infrastructure::database::open_database;
use infrastructure::logging::init_logging;
use infrastructure::paths::{AppPaths, StorageMode};
use infrastructure::settings::JsonSettingsRepository;
use infrastructure::wishes::SqliteWishRepository;

use crate::commands;
use crate::error::StartupError;
use crate::state::AppState;
use crate::window;

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

    let state = build_state(&paths)?;
    let webview_dir = paths.webview_dir.clone();

    tauri::Builder::default()
        .manage(state)
        // `setup` выполняется один раз после запуска Tauri, до показа окон.
        // `move` переносит `webview_dir` внутрь замыкания.
        .setup(move |app| window::create_main_window(app, webview_dir))
        .invoke_handler(tauri::generate_handler![
            commands::games::list_games,
            commands::games::get_game_statistics,
            commands::reference::get_reference_summary,
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())?;

    tracing::info!("Wish Counter завершил работу");
    Ok(())
}

/// Создаёт сервисы приложения и связывает их с реализациями.
///
/// Возвращает ошибку, если не удалось открыть базу данных.
fn build_state(paths: &AppPaths) -> Result<AppState, StartupError> {
    let settings_repository = Arc::new(JsonSettingsRepository::new(&paths.settings_file));
    let wish_repository = Arc::new(SqliteWishRepository::new(open_database(
        &paths.database_file,
    )?));
    let game_catalog = Arc::new(GameCatalog::new().with_module(Arc::new(GenshinModule)));

    Ok(AppState {
        reference_service: ReferenceService::from_catalog(&game_catalog),
        statistics_service: StatisticsService::new(Arc::clone(&game_catalog), wish_repository),
        game_catalog,
        settings_service: SettingsService::new(settings_repository),
    })
}
