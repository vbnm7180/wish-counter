// Локализация интерфейса.
//
// Чтобы добавить язык:
// 1. создать locales/<код>.ts по образцу en.ts;
// 2. добавить его в DICTIONARIES и LANGUAGES ниже;
// 3. добавить вариант в domain::settings::Language (Rust) и LanguageCode (api/types.ts).
import type { LanguageCode } from '../api/types';
import { en } from './locales/en';
import { ru, type TranslationKey } from './locales/ru';
import { interpolate, type TranslationParams } from './interpolate';

export type { TranslationKey };

/** Словари всех языков. */
const DICTIONARIES: Record<LanguageCode, Record<TranslationKey, string>> = { ru, en };

/** Языки для выбора в настройках, с ключом подписи. */
export const LANGUAGES: { code: LanguageCode; labelKey: TranslationKey }[] = [
  { code: 'ru', labelKey: 'language.ru' },
  { code: 'en', labelKey: 'language.en' },
];

/**
 * Текущий язык и функции перевода.
 *
 * Поле `language` объявлено через руну `$state`, поэтому всё,
 * что вызывает `t()` в разметке, автоматически перерисуется при смене языка.
 */
class I18n {
  language = $state<LanguageCode>('ru');

  /**
   * Возвращает строку на текущем языке (или сам ключ, если строки нет).
   * Параметры подставляются вместо {имя}: t('game.pity_progress', { current: 5, hard: 90 }).
   */
  t = (key: TranslationKey, params?: TranslationParams): string =>
    interpolate(DICTIONARIES[this.language][key] ?? key, params);

  /** Переводит название игры по её идентификатору (ключ games.<id>.name). */
  gameName = (gameId: string): string => this.lookup(`games.${gameId}.name`, gameId);

  /** Переводит название пула баннеров (ключ <игра>.banner.<пул>). */
  poolName = (gameId: string, poolId: string): string => this.lookup(`${gameId}.banner.${poolId}`, poolId);

  /** Ищет строку по ключу, собранному во время выполнения; если её нет — возвращает запасной текст. */
  private lookup(key: string, fallback: string): string {
    return DICTIONARIES[this.language][key as TranslationKey] ?? fallback;
  }

  /** Меняет язык и обновляет атрибут lang у <html> (для доступности). */
  setLanguage(language: LanguageCode): void {
    this.language = language;
    document.documentElement.lang = language;
  }
}

/** Единственный экземпляр локализации для всего интерфейса. */
export const i18n = new I18n();
