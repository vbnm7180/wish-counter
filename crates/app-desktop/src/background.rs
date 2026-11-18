//! Фоновые задачи, которые запускаются вместе с приложением.

use std::sync::Arc;

use application::services::{ReferenceUpdater, UpdateOutcome};
use tauri::AppHandle;

use crate::events;

/// Имя фонового потока проверки обновлений (видно в отладчике и логах).
const REFERENCE_CHECK_THREAD_NAME: &str = "reference-update";

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
