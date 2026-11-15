// Тесты форматирования чисел (запуск: npm test).
import { describe, expect, it } from 'vitest';
import { formatChance, formatNumber } from './format';

/** Заменяет любые пробелы (в т.ч. неразрывные) обычным, чтобы сравнивать строки. */
function normalizeSpaces(text: string): string {
  return text.replace(/\s/g, ' ');
}

describe('formatNumber', () => {
  it('разделяет разряды по-русски пробелом', () => {
    expect(normalizeSpaces(formatNumber(65920, 'ru'))).toBe('65 920');
  });

  it('разделяет разряды по-английски запятой', () => {
    expect(formatNumber(65920, 'en')).toBe('65,920');
  });
});

describe('formatChance', () => {
  it('переводит сотые доли процента в проценты', () => {
    expect(formatChance(5500, 'en')).toBe('55%');
    expect(formatChance(10000, 'en')).toBe('100%');
  });

  it('использует десятичный разделитель языка', () => {
    expect(formatChance(3750, 'ru')).toBe('37,5%');
    expect(formatChance(3750, 'en')).toBe('37.5%');
  });
});
