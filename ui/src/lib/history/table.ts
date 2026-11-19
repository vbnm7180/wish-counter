// Логика таблицы молитв: фильтры и постраничный вывод.
// Вынесена из компонента, чтобы её можно было проверить тестами.
import type { WishRow } from '../api/types';

/** Сколько строк показывать на одной странице. */
export const PAGE_SIZE = 50;

/** Тип предмета для фильтра: все, только персонажи или только оружие. */
export type KindFilter = 'all' | 'character' | 'weapon';

/** Условия фильтра таблицы. */
export interface RowFilter {
  /** Какие редкости показывать. */
  rarities: Set<3 | 4 | 5>;
  /** Какие предметы показывать. */
  kind: KindFilter;
  /** Часть названия (без учёта регистра); пустая строка — без поиска. */
  search: string;
}

/** Фильтр по умолчанию: все редкости, все типы, без поиска. */
export function defaultFilter(): RowFilter {
  return { rarities: new Set([3, 4, 5]), kind: 'all', search: '' };
}

/**
 * Оставляет строки, подходящие под фильтр. `nameOf` возвращает название
 * строки на текущем языке — по нему идёт поиск.
 */
export function filterRows(rows: WishRow[], filter: RowFilter, nameOf: (row: WishRow) => string): WishRow[] {
  const query = filter.search.trim().toLocaleLowerCase();
  return rows.filter(
    (row) =>
      filter.rarities.has(row.rarity) &&
      (filter.kind === 'all' || row.item_kind === filter.kind) &&
      (query === '' || nameOf(row).toLocaleLowerCase().includes(query)),
  );
}

/** Сколько страниц нужно для `total` строк (минимум одна — пустая). */
export function pageCount(total: number, pageSize = PAGE_SIZE): number {
  return Math.max(1, Math.ceil(total / pageSize));
}

/** Строки страницы `page` (с 1). Номер за пределами — ближайшая существующая страница. */
export function pageRows<T>(rows: T[], page: number, pageSize = PAGE_SIZE): T[] {
  const safePage = Math.min(Math.max(1, page), pageCount(rows.length, pageSize));
  const start = (safePage - 1) * pageSize;
  return rows.slice(start, start + pageSize);
}
