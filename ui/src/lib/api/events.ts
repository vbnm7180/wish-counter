// События, которые присылает Rust-ядро (см. crates/app-desktop/src/events.rs).
//
// Команды — «интерфейс спрашивает, ядро отвечает». События — наоборот:
// ядро само сообщает, что что-то изменилось (например, справочник обновился в фоне).
import { isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

/** Имя события «справочник игры обновлён» (совпадает с REFERENCE_UPDATED в Rust). */
const REFERENCE_UPDATED = 'reference-updated';

/** Данные события «справочник обновлён». */
interface ReferenceUpdatedPayload {
  gameId: string;
}

/** Функция отписки от события. */
export type Unsubscribe = () => void;

/**
 * Подписывается на обновление справочника. Возвращает функцию отписки —
 * её нужно вызвать, когда компонент исчезает со страницы.
 * В браузере (без Tauri) событий нет — подписка ничего не делает.
 */
export async function onReferenceUpdated(callback: (gameId: string) => void): Promise<Unsubscribe> {
  if (!isTauri()) return () => {};
  return listen<ReferenceUpdatedPayload>(REFERENCE_UPDATED, (event) => callback(event.payload.gameId));
}
