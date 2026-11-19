// Адреса картинок, которые отдаёт Rust через собственный протокол `wcimg`
// (crates/app-desktop/src/image_protocol):
// - /items/<игра>/<предмет> — картинка предмета;
// - /games/<игра>/icon     — иконка игры.
// Формат адреса зависит от ОС: в Windows (WebView2) — http://wcimg.localhost/<путь>,
// в остальных — wcimg://localhost/<путь>.
import { isTauri } from '@tauri-apps/api/core';

/** Имя протокола (совпадает с IMAGE_SCHEME в Rust). */
const IMAGE_SCHEME = 'wcimg';

/** Строит адрес по частям пути для указанной ОС. Чистая функция — удобно тестировать. */
export function buildImageUrl(parts: string[], isWindows: boolean): string {
  const path = parts.map(encodeURIComponent).join('/');
  return isWindows ? `http://${IMAGE_SCHEME}.localhost/${path}` : `${IMAGE_SCHEME}://localhost/${path}`;
}

/**
 * Адрес по частям пути или null, если интерфейс запущен в обычном браузере
 * (`npm run dev`): там протокола нет, и компонент покажет заглушку.
 */
function imageUrl(parts: string[]): string | null {
  if (!isTauri()) return null;
  return buildImageUrl(parts, navigator.userAgent.includes('Windows'));
}

/** Адрес картинки предмета. */
export function itemImageUrl(gameId: string, itemId: string): string | null {
  return imageUrl(['items', gameId, itemId]);
}

/** Адрес иконки игры. */
export function gameIconUrl(gameId: string): string | null {
  return imageUrl(['games', gameId, 'icon']);
}
