// Адреса картинок предметов.
//
// Картинки отдаёт Rust через собственный протокол `wcimg`
// (crates/app-desktop/src/image_protocol). Формат адреса зависит от ОС:
// в Windows (WebView2) — http://wcimg.localhost/<игра>/<предмет>,
// в остальных — wcimg://localhost/<игра>/<предмет>.
import { isTauri } from '@tauri-apps/api/core';

/** Имя протокола (совпадает с IMAGE_SCHEME в Rust). */
const IMAGE_SCHEME = 'wcimg';

/** Строит адрес картинки для указанной ОС. Чистая функция — удобно тестировать. */
export function buildItemImageUrl(gameId: string, itemId: string, isWindows: boolean): string {
  const path = `${encodeURIComponent(gameId)}/${encodeURIComponent(itemId)}`;
  return isWindows ? `http://${IMAGE_SCHEME}.localhost/${path}` : `${IMAGE_SCHEME}://localhost/${path}`;
}

/**
 * Адрес картинки предмета или null, если интерфейс запущен в обычном браузере
 * (`npm run dev`): там протокола нет, и компонент покажет заглушку.
 */
export function itemImageUrl(gameId: string, itemId: string): string | null {
  if (!isTauri()) return null;
  return buildItemImageUrl(gameId, itemId, navigator.userAgent.includes('Windows'));
}
