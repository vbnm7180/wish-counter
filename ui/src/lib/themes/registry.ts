// Реестр тем оформления.
//
// Тема — это набор CSS-переменных (цветов) в отдельном файле.
// Чтобы добавить тему:
// 1. создать <id>.css по образцу dark.css с селектором :root[data-theme='<id>'];
// 2. подключить его в ui/src/main.ts;
// 3. добавить запись в THEMES ниже и подпись 'theme.<id>' в словари.
// Rust-код менять не нужно: ядро хранит тему как строку.
import type { TranslationKey } from '../i18n/index.svelte';

/** Описание темы для меню выбора. */
export interface ThemeDefinition {
  /** Идентификатор (значение атрибута data-theme). */
  id: string;
  /** Ключ подписи темы в словаре. */
  labelKey: TranslationKey;
}

/** Все доступные темы. */
export const THEMES: ThemeDefinition[] = [
  { id: 'dark', labelKey: 'theme.dark' },
  { id: 'light', labelKey: 'theme.light' },
];

/** Тема, которая используется, если выбранная не найдена. */
export const DEFAULT_THEME_ID = 'dark';

/**
 * Применяет тему к странице, выставляя атрибут data-theme у <html>.
 * Неизвестная тема заменяется темой по умолчанию.
 */
export function applyTheme(themeId: string): void {
  const exists = THEMES.some((theme) => theme.id === themeId);
  document.documentElement.dataset.theme = exists ? themeId : DEFAULT_THEME_ID;
}
