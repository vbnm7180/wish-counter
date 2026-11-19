// Тесты сообщений о результате импорта (запуск: npm test).
import { describe, expect, it } from 'vitest';
import type { ImportSummary } from '../api/types';
import { importErrorKey, summaryLines } from './import-messages';

/** Итог импорта с переопределёнными полями. */
function summary(overrides: Partial<ImportSummary> = {}): ImportSummary {
  return { total_in_file: 10, added: 10, already_present: 0, duplicates_skipped: 0, server_region: null, ...overrides };
}

describe('summaryLines', () => {
  it('для первого импорта — только число добавленных', () => {
    expect(summaryLines(summary(), String).map((line) => line.key)).toEqual(['import.result_added']);
  });

  it('добавляет строки о повторах, уже сохранённых и сервере', () => {
    const regionName = (region: string) => (region === 'europe' ? 'Европа' : region);
    const lines = summaryLines(
      summary({ added: 2, already_present: 8, duplicates_skipped: 5, server_region: 'europe' }),
      regionName,
    );

    expect(lines.map((line) => line.key)).toEqual([
      'import.result_added',
      'import.result_already_present',
      'import.result_duplicates',
      'import.result_region',
    ]);
    expect(lines[3].params).toEqual({ region: 'Европа' });
  });
});

describe('importErrorKey', () => {
  it('различает известные ошибки', () => {
    expect(importErrorKey({ code: 'import_invalid_file', message: '' })).toBe('import.error_invalid_file');
    expect(importErrorKey({ code: 'import_unknown_items', message: '' })).toBe('import.error_unknown_items');
  });

  it('для остальных — общий текст', () => {
    expect(importErrorKey({ code: 'storage_io', message: '' })).toBe('import.error_failed');
    expect(importErrorKey('boom')).toBe('import.error_failed');
  });
});
