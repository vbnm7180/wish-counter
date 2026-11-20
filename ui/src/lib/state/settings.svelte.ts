// Состояние пользовательских настроек в интерфейсе.
//
// Отвечает за то, чтобы настройки из Rust-ядра применялись к странице
// (тема, язык, фон) и чтобы изменения сохранялись обратно в ядро.
import { backend } from '../api/backend';
import type { AppSettings } from '../api/types';
import { applyBackground } from '../backgrounds/background';
import { i18n, type TranslationKey } from '../i18n/index.svelte';
import { backgroundImageUrl } from '../images/item-image';
import { applyTheme, DEFAULT_THEME_ID } from '../themes/registry';

/** Настройки до загрузки из ядра. */
const INITIAL_SETTINGS: AppSettings = { theme: DEFAULT_THEME_ID, language: 'ru', background: null };

class SettingsState {
  /** Текущие применённые настройки. */
  current = $state<AppSettings>(INITIAL_SETTINGS);
  /** Ключ текста последней ошибки или null. */
  error = $state<TranslationKey | null>(null);

  /** Загружает настройки из ядра и применяет их. */
  async load(): Promise<void> {
    try {
      this.apply(await backend.getSettings());
    } catch (cause) {
      console.error('Не удалось загрузить настройки', cause);
      this.error = 'error.load_failed';
    }
  }

  /**
   * Меняет часть настроек.
   * Изменение применяется сразу («оптимистично»), а если сохранить
   * не удалось — откатывается к прежнему значению.
   */
  async update(changes: Partial<AppSettings>): Promise<void> {
    const previous = this.current;
    const next = { ...previous, ...changes };
    this.apply(next);
    try {
      this.apply(await backend.updateSettings(next));
      this.error = null;
    } catch (cause) {
      console.error('Не удалось сохранить настройки', cause);
      this.apply(previous);
      this.error = 'settings.save_error';
    }
  }

  /** Применяет настройки, уже сохранённые ядром (например, после удаления фона). */
  replace(settings: AppSettings): void {
    this.apply(settings);
  }

  /** Запоминает настройки и применяет тему, язык и фон к странице. */
  private apply(settings: AppSettings): void {
    this.current = settings;
    applyTheme(settings.theme);
    i18n.setLanguage(settings.language);
    applyBackground(settings.background ? backgroundImageUrl(settings.background) : null);
  }
}

/** Единственный экземпляр состояния настроек. */
export const settingsState = new SettingsState();
