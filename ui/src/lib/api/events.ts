// События, которые присылает Rust-ядро (см. crates/app-desktop/src/events.rs).
//
// Команды — «интерфейс спрашивает, ядро отвечает». События — наоборот:
// ядро само сообщает, что что-то изменилось (например, справочник обновился в фоне).
import { isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

/** Имя события «справочник игры обновлён» (совпадает с REFERENCE_UPDATED в Rust). */
const REFERENCE_UPDATED = 'reference-updated';
/** Имя события «история молитв изменилась» (совпадает с HISTORY_UPDATED в Rust). */
const HISTORY_UPDATED = 'history-updated';

/** Данные событий об игре. */
interface GamePayload {
  gameId: string;
}

/** Функция отписки от события. */
export type Unsubscribe = () => void;

/**
 * Подписывается на обновление справочника. Возвращает функцию отписки —
 * её нужно вызвать, когда компонент исчезает со страницы.
 * В браузере (без Tauri) событий нет — подписка ничего не делает.
 */
export function onReferenceUpdated(callback: (gameId: string) => void): Promise<Unsubscribe> {
  return onGameEvent(REFERENCE_UPDATED, callback);
}

/** Подписывается на изменение истории молитв (например, после импорта). */
export function onHistoryUpdated(callback: (gameId: string) => void): Promise<Unsubscribe> {
  return onGameEvent(HISTORY_UPDATED, callback);
}

/** Подписка на событие с идентификатором игры. В браузере (без Tauri) ничего не делает. */
async function onGameEvent(name: string, callback: (gameId: string) => void): Promise<Unsubscribe> {
  if (!isTauri()) return () => {};
  return listen<GamePayload>(name, (event) => callback(event.payload.gameId));
}
