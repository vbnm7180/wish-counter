// Тесты логики таблицы молитв (запуск: npm test).
import { describe, expect, it } from 'vitest';
import type { WishRow } from '../api/types';
import { defaultFilter, filterRows, pageCount, pageRows } from './table';

/** Строка таблицы с переопределёнными полями. */
function row(overrides: Partial<WishRow>): WishRow {
  return {
    id: '1',
    time: '2026-01-01 00:00:00',
    item_id: 'cool_steel',
    name: { en: 'Cool Steel', ru: 'Холодное лезвие' },
    item_kind: 'weapon',
    rarity: 3,
    pity: null,
    outcome: null,
    banner_name: null,
    ...overrides,
  };
}

/** Название на русском. */
const russian = (item: WishRow) => item.name?.ru ?? item.item_id;

const ROWS = [
  row({ id: 'a' }),
  row({ id: 'b', item_id: 'mitya', item_kind: 'character', rarity: 5, name: { en: 'Mitya', ru: 'Митя' } }),
  row({ id: 'c', item_id: 'the_flute', rarity: 4, name: { en: 'The Flute', ru: 'Флейта' } }),
];

describe('filterRows', () => {
  it('по умолчанию показывает всё', () => {
    expect(filterRows(ROWS, defaultFilter(), russian)).toHaveLength(3);
  });

  it('фильтрует по редкости', () => {
    const filter = { ...defaultFilter(), rarities: new Set<3 | 4 | 5>([4, 5]) };
    expect(filterRows(ROWS, filter, russian).map((item) => item.id)).toEqual(['b', 'c']);
  });

  it('фильтрует по типу предмета', () => {
    const filter = { ...defaultFilter(), kind: 'character' as const };
    expect(filterRows(ROWS, filter, russian).map((item) => item.id)).toEqual(['b']);
  });

  it('ищет по части названия без учёта регистра', () => {
    const filter = { ...defaultFilter(), search: '  мит ' };
    expect(filterRows(ROWS, filter, russian).map((item) => item.id)).toEqual(['b']);
  });
});

describe('постраничный вывод', () => {
  it('считает число страниц', () => {
    expect(pageCount(0)).toBe(1);
    expect(pageCount(50)).toBe(1);
    expect(pageCount(51)).toBe(2);
  });

  it('отдаёт строки нужной страницы', () => {
    const numbers = Array.from({ length: 120 }, (_, index) => index);
    expect(pageRows(numbers, 2)).toEqual(numbers.slice(50, 100));
    expect(pageRows(numbers, 3)).toHaveLength(20);
  });

  it('номер за пределами приводит к ближайшей странице', () => {
    const numbers = Array.from({ length: 60 }, (_, index) => index);
    expect(pageRows(numbers, 99)).toEqual(numbers.slice(50));
    expect(pageRows(numbers, 0)).toEqual(numbers.slice(0, 50));
  });
});
