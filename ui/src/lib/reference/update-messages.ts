// Тексты результата обновления справочника (кнопка «Обновить данные»).
// Логика вынесена из компонента, чтобы её можно было проверить тестами.
import type { CommandError, UpdateOutcome } from '../api/types';
import type { TranslationKey } from '../i18n/locales/ru';

/** Сообщение для успешного результата обновления. */
export function outcomeMessageKey(outcome: UpdateOutcome): TranslationKey {
  switch (outcome.status) {
    case 'updated':
      return 'reference.updated';
    case 'up_to_date':
    case 'not_due':
      return 'reference.up_to_date';
    case 'unsupported':
      return 'reference.unsupported';
  }
}

/** Коды ошибок ядра, для которых есть отдельные сообщения. */
const ERROR_MESSAGES: Record<string, TranslationKey> = {
  reference_download_failed: 'reference.download_failed',
  reference_invalid: 'reference.invalid',
};

/**
 * Сообщение для ошибки обновления. Ошибка из Rust приходит объектом
 * `{ code, message }`; всё остальное (например, сбой вызова) — общий текст.
 */
export function errorMessageKey(error: unknown): TranslationKey {
  const code = isCommandError(error) ? error.code : '';
  return ERROR_MESSAGES[code] ?? 'reference.update_failed';
}

/** Проверяет, что значение похоже на ошибку команды Rust. */
function isCommandError(value: unknown): value is CommandError {
  return typeof value === 'object' && value !== null && 'code' in value && typeof value.code === 'string';
}
