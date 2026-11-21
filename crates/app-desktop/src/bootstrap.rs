//! Сборка приложения (composition root).
//!
//! Это единственное место, где создаются конкретные реализации
//! (файловое хранилище, база данных, игровые модули) и передаются в сервисы.
//! Остальной код зависит только от абстракций (трейтов).

use std::sync::Arc;

use tauri::Manager;

use application::services::{
    BackgroundService, ClearGameService, ExportService, GameApiImportService, GameCatalog,
    HistoryService, ImageService, ImportService, ProfileService, ReferenceService,
    ReferenceUpdater, SettingsService, StatisticsService,
};
use game_genshin::GenshinModule;
use infrastructure::backgrounds::FileBackgroundStore;
use infrastructure::database::{SharedConnection, open_database};
use infrastructure::image_cache::FileImageCache;
use infrastructure::logging::init_logging;
use infrastructure::paths::{AppPaths, StorageMode};
use infrastructure::profiles::SqliteProfileRepository;
use infrastructure::reference_cache::JsonReferenceCache;
use infrastructure::remote::{GameApiHttpClient, GitHubReferenceDownloader, HttpClient};
use infrastructure::settings::JsonSettingsRepository;
use infrastructure::wishes::SqliteWishRepository;

use crate::background;
use crate::commands;
use crate::error::StartupError;
use crate::image_protocol::{self, IMAGE_SCHEME};
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
        // Плагин системных диалогов; используется только из Rust (экспорт, выбор фона).
        .plugin(tauri_plugin_dialog::init())
        // `setup` выполняется один раз после запуска Tauri, до показа окон.
        // `move` переносит `webview_dir` внутрь замыкания.
        .setup(move |app| {
            window::create_main_window(app, webview_dir)?;
            // `app.state::<AppState>()` достаёт состояние, переданное в `manage`.
            let updater = Arc::clone(&app.state::<AppState>().reference_updater);
            background::spawn_reference_check(app.handle().clone(), updater);
            Ok(())
        })
        // Картинки предметов и фонов: адреса wcimg://... обрабатывает Rust (см. image_protocol.rs).
        .register_asynchronous_uri_scheme_protocol(IMAGE_SCHEME, |context, request, responder| {
            image_protocol::handle_request(context.app_handle(), &request, responder);
        })
        .invoke_handler(tauri::generate_handler![
            commands::backgrounds::list_backgrounds,
            commands::backgrounds::add_background,
            commands::backgrounds::remove_background,
            commands::game_import::import_from_game,
            commands::game_import::clear_game_data,
            commands::games::list_games,
            commands::games::get_game_statistics,
            commands::games::get_game_profile,
            commands::games::set_player_uid,
            commands::games::set_server_region,
            commands::history::import_history,
            commands::history::export_history,
            commands::history::get_pool_history,
            commands::reference::get_reference_summary,
            commands::reference::update_reference,
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
    let settings_service = Arc::new(SettingsService::new(Arc::new(JsonSettingsRepository::new(
        &paths.settings_file,
    ))));
    let background_service = Arc::new(BackgroundService::new(
        Arc::new(FileBackgroundStore::new(&paths.backgrounds_dir)),
        Arc::clone(&settings_service),
    ));
    // Одно соединение с базой на все хранилища.
    let database = SharedConnection::new(open_database(&paths.database_file)?);
    let wish_repository = Arc::new(SqliteWishRepository::new(database.clone()));
    let profile_repository = Arc::new(SqliteProfileRepository::new(database));

    let game_catalog = Arc::new(GameCatalog::new().with_module(Arc::new(GenshinModule)));
    let reference_cache = Arc::new(JsonReferenceCache::new(&paths.catalog_dir));
    // Один загрузчик GitHub и для справочника, и для картинок (реализует оба порта).
    let github = Arc::new(GitHubReferenceDownloader::new(HttpClient::new()));
    let reference_service = Arc::new(ReferenceService::new(
        &game_catalog,
        reference_cache.as_ref(),
    ));
    let reference_updater = Arc::new(ReferenceUpdater::new(
        Arc::clone(&game_catalog),
        Arc::clone(&reference_service),
        reference_cache,
        github.clone(),
    ));
    let image_service = Arc::new(ImageService::new(
        Arc::clone(&game_catalog),
        Arc::clone(&reference_service),
        Arc::new(FileImageCache::new(&paths.images_dir)),
        github,
    ));

    Ok(AppState {
        statistics_service: StatisticsService::new(
            Arc::clone(&game_catalog),
            wish_repository.clone(),
            Arc::clone(&reference_service),
            profile_repository.clone(),
        ),
        export_service: Arc::new(ExportService::new(
            Arc::clone(&game_catalog),
            Arc::clone(&reference_service),
            wish_repository.clone(),
            profile_repository.clone(),
        )),
        history_service: Arc::new(HistoryService::new(
            Arc::clone(&game_catalog),
            Arc::clone(&reference_service),
            wish_repository.clone(),
            profile_repository.clone(),
        )),
        import_service: Arc::new(ImportService::new(
            Arc::clone(&game_catalog),
            Arc::clone(&reference_service),
            wish_repository.clone(),
            profile_repository.clone(),
        )),
        game_api_import_service: Arc::new(GameApiImportService::new(
            Arc::clone(&game_catalog),
            Arc::clone(&reference_service),
            wish_repository.clone(),
            Arc::new(GameApiHttpClient::new()),
        )),
        clear_game_service: Arc::new(ClearGameService::new(
            Arc::clone(&game_catalog),
            wish_repository.clone(),
            profile_repository.clone(),
        )),
        profile_service: ProfileService::new(Arc::clone(&game_catalog), profile_repository),
        reference_updater,
        image_service,
        game_catalog,
        settings_service,
        background_service,
    })
}
