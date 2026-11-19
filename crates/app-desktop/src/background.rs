//! Фоновые задачи, которые запускаются вместе с приложением.

use std::sync::Arc;

use application::services::{ImageService, ReferenceUpdater, UpdateOutcome};
use domain::game::GameId;
use domain::wish::ItemId;
use tauri::AppHandle;

use crate::events;

/// Имя фонового потока проверки обновлений (видно в отладчике и логах).
const REFERENCE_CHECK_THREAD_NAME: &str = "reference-update";
/// Имя фонового потока докачки картинок.
const IMAGE_PREFETCH_THREAD_NAME: &str = "image-prefetch";

/// Запускает проверку обновлений справочников в отдельном потоке.
///
/// Окно открывается сразу, не дожидаясь сети. Если справочник обновился,
/// интерфейс получает событие и перечитывает данные. Без интернета проверка
/// просто завершится с записью в лог — приложение работает со своим справочником.
pub fn spawn_reference_check(app: AppHandle, updater: Arc<ReferenceUpdater>) {
    let spawned = std::thread::Builder::new()
        .name(REFERENCE_CHECK_THREAD_NAME.into())
        .spawn(move || {
            let now = chrono::Utc::now().naive_utc();
            for (game_id, result) in updater.update_all_due(now) {
                if matches!(result, Ok(UpdateOutcome::Updated { .. })) {
                    events::emit_reference_updated(&app, &game_id);
                }
            }
        });
    if let Err(error) = spawned {
        tracing::error!(%error, "не удалось запустить фоновую проверку справочников");
    }
}

/// Докачивает в фоне картинки предметов (после импорта истории), чтобы потом
/// они показывались без интернета. Уже скачанные берутся из кэша мгновенно.
pub fn spawn_image_prefetch(service: Arc<ImageService>, game_id: GameId, item_ids: Vec<ItemId>) {
    let spawned = std::thread::Builder::new()
        .name(IMAGE_PREFETCH_THREAD_NAME.into())
        .spawn(move || {
            let cached = service.prefetch(&game_id, &item_ids);
            tracing::info!(game = %game_id, requested = item_ids.len(), cached, "картинки докачаны");
        });
    if let Err(error) = spawned {
        tracing::error!(%error, "не удалось запустить докачку картинок");
    }
}
