// Фон приложения: применение выбранной картинки к странице и тексты ошибок добавления.
//
// Картинка задаётся CSS-переменной --app-background-image (см. themes/base.css),
// а атрибут data-background="on" у <html> делает панели полупрозрачными,
// чтобы фон был виден, а текст оставался читаемым.
import type { TranslationKey } from '../i18n/locales/ru';

/** Имя CSS-переменной с картинкой фона. */
const BACKGROUND_VARIABLE = '--app-background-image';

/** Значение CSS-переменной: url("...") или none (без фона). */
export function backgroundCssValue(url: string | null): string {
  return url ? `url("${url}")` : 'none';
}

/**
 * Применяет фон к странице. `url` — адрес картинки (протокол wcimg) или null — без фона.
 * `root` можно подменить в тестах.
 */
export function applyBackground(url: string | null, root: HTMLElement = document.documentElement): void {
  root.style.setProperty(BACKGROUND_VARIABLE, backgroundCssValue(url));
  if (url) {
    root.dataset.background = 'on';
  } else {
    delete root.dataset.background;
  }
}

/** Коды ошибок ядра при добавлении фона, для которых есть отдельные сообщения. */
const ERROR_MESSAGES: Record<string, TranslationKey> = {
  background_too_large: 'background.error_too_large',
  background_unsupported: 'background.error_unsupported',
  background_read_failed: 'background.error_read',
};

/** Сообщение об ошибке добавления фона по коду ошибки ядра (`{ code, message }`). */
export function backgroundErrorKey(error: unknown): TranslationKey {
  const code =
    typeof error === 'object' && error !== null && 'code' in error && typeof error.code === 'string'
      ? error.code
      : '';
  return ERROR_MESSAGES[code] ?? 'background.error_failed';
}
