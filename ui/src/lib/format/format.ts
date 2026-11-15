// Форматирование чисел для отображения с учётом языка интерфейса.
import type { LanguageCode } from '../api/types';

/** Локаль Intl для каждого языка интерфейса. */
const LOCALES: Record<LanguageCode, string> = { ru: 'ru-RU', en: 'en-US' };

/** Сотых долей процента в одном проценте (5500 → 55%). */
const BASIS_POINTS_PER_PERCENT = 100;

/** Сколько знаков после запятой показывать у процентов. */
const MAX_PERCENT_FRACTION_DIGITS = 2;

/**
 * Форматирует целое число с разделителями разрядов:
 * 12800 → «12 800» (ru) или «12,800» (en).
 */
export function formatNumber(value: number, language: LanguageCode): string {
  return new Intl.NumberFormat(LOCALES[language]).format(value);
}

/**
 * Форматирует вероятность из сотых долей процента:
 * 5500 → «55%», 3750 → «37,5%» (ru) или «37.5%» (en).
 */
export function formatChance(basisPoints: number, language: LanguageCode): string {
  const percent = new Intl.NumberFormat(LOCALES[language], {
    maximumFractionDigits: MAX_PERCENT_FRACTION_DIGITS,
  }).format(basisPoints / BASIS_POINTS_PER_PERCENT);
  return `${percent}%`;
}
