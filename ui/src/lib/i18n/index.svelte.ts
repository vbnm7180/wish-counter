// Локализация интерфейса.
//
// Чтобы добавить язык:
// 1. создать locales/<код>.ts по образцу en.ts;
// 2. добавить его в DICTIONARIES и LANGUAGES ниже;
// 3. добавить вариант в domain::settings::Language (Rust) и LanguageCode (api/types.ts).
import type { LanguageCode } from '../api/types';
import { en } from './locales/en';
import { ru, type TranslationKey } from './locales/ru';

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

  /** Возвращает строку на текущем языке (или сам ключ, если строки нет). */
  t = (key: TranslationKey): string => DICTIONARIES[this.language][key] ?? key;

  /** Переводит название игры по её идентификатору (ключ games.<id>.name). */
  gameName = (gameId: string): string => {
    const key = `games.${gameId}.name` as TranslationKey;
    return DICTIONARIES[this.language][key] ?? gameId;
  };

  /** Меняет язык и обновляет атрибут lang у <html> (для доступности). */
  setLanguage(language: LanguageCode): void {
    this.language = language;
    document.documentElement.lang = language;
  }
}

/** Единственный экземпляр локализации для всего интерфейса. */
export const i18n = new I18n();
