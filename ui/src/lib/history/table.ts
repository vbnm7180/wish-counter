// Логика таблицы молитв: фильтры и постраничный вывод.
// Вынесена из компонента, чтобы её можно было проверить тестами.
import type { WishRow } from '../api/types';

/** Сколько строк показывать на одной странице. */
export const PAGE_SIZE = 50;

/**
 * Жёсткий гарант 5★ по пулу — шкала для окраски (как в Paimon.moe).
 * Оружейный баннер — 80, остальные — 90.
 */
export const HARD_PITY_5: Record<string, number> = {
  character:  90,
  chronicled: 90,
  standard:   90,
  beginner:   90,
  weapon:     80,
};

/**
 * Оттенок цвета номера по гаранту (HSL hue, 0–120).
 * Формула как в Paimon.moe: линейная от красного (0) к зелёному (120).
 * hardPity — жёсткий гарант пула (90 или 80 для оружия), для 4★ = 10.
 * null — не окрашивать (нет номера или 3★).
 */
export function pityHue(
  pity: number | null,
  rarity: 3 | 4 | 5,
  hardPity?: number,
): number | null {
  if (pity === null || rarity === 3) return null;
  const scale = hardPity ?? (rarity === 5 ? 90 : 10);
  const share = Math.min(Math.max((scale - pity) / scale, 0), 1);
  return Math.round(share * 120);
}

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

/**
 * Дата и время молитвы для таблицы: «2026-11-19 20:14:05» → «19.11.2026 20:14».
 * Время — по серверу игры, как его отдаёт игра; часовой пояс не пересчитывается.
 */
export function formatWishTime(time: string): string {
  const [date = '', clock = ''] = time.split(' ');
  const [year, month, day] = date.split('-');
  return `${day}.${month}.${year} ${clock.slice(0, 5)}`;
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
