// Тексты результата импорта истории. Логика вынесена из компонента, чтобы её
// можно было проверить тестами.
import type { ImportSummary, ServerRegion } from '../api/types';
import type { TranslationKey } from '../i18n/locales/ru';
import type { TranslationParams } from '../i18n/interpolate';

/** Одна строка сообщения: ключ перевода и параметры. */
export interface MessageLine {
  key: TranslationKey;
  params?: TranslationParams;
}

/**
 * Строки отчёта об успешном импорте. Необязательные строки — только если есть что сказать.
 * `regionName` переводит сервер в название на языке интерфейса.
 */
export function summaryLines(summary: ImportSummary, regionName: (region: ServerRegion) => string): MessageLine[] {
  const lines: MessageLine[] = [
    { key: 'import.result_added', params: { added: summary.added, total: summary.total_in_file } },
  ];
  if (summary.already_present > 0) {
    lines.push({ key: 'import.result_already_present', params: { count: summary.already_present } });
  }
  if (summary.duplicates_skipped > 0) {
    lines.push({ key: 'import.result_duplicates', params: { count: summary.duplicates_skipped } });
  }
  if (summary.server_region) {
    lines.push({ key: 'import.result_region', params: { region: regionName(summary.server_region) } });
  }
  if (summary.uid) {
    lines.push({ key: 'import.result_uid', params: { uid: summary.uid } });
  }
  return lines;
}

/** Коды ошибок экспорта, для которых есть отдельные сообщения. */
const EXPORT_ERROR_MESSAGES: Record<string, TranslationKey> = {
  export_empty_history: 'export.error_empty',
  export_write_failed: 'export.error_write',
};

/** Сообщение об ошибке экспорта по коду ошибки ядра. */
export function exportErrorKey(error: unknown): TranslationKey {
  return EXPORT_ERROR_MESSAGES[errorCode(error)] ?? 'export.error_failed';
}

/** Код ошибки команды Rust (`{ code, message }`); пустая строка — не ошибка команды. */
function errorCode(error: unknown): string {
  return typeof error === 'object' && error !== null && 'code' in error && typeof error.code === 'string'
    ? error.code
    : '';
}

/** Коды ошибок ядра, для которых есть отдельные сообщения. */
const ERROR_MESSAGES: Record<string, TranslationKey> = {
  import_invalid_file: 'import.error_invalid_file',
  import_unknown_items: 'import.error_unknown_items',
  import_file_too_large: 'import.error_too_large',
};

/** Сообщение об ошибке импорта по коду ошибки ядра. */
export function importErrorKey(error: unknown): TranslationKey {
  return ERROR_MESSAGES[errorCode(error)] ?? 'import.error_failed';
}

/** Коды ошибок импорта из API игры. */
const GAME_IMPORT_ERROR_MESSAGES: Record<string, TranslationKey> = {
  game_import_failed: 'import.error_game_import_failed',
  authkey_expired: 'import.error_authkey_expired',
};

/** Сообщение об ошибке импорта из игры по коду ошибки ядра. */
export function gameImportErrorKey(error: unknown): TranslationKey {
  return GAME_IMPORT_ERROR_MESSAGES[errorCode(error)] ?? 'import.error_game_failed';
}
