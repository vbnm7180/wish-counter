// Подготовка статуса 50/50 к отображению.
//
// Rust уже посчитал всё нужное; здесь только выбирается, какие строки
// показать в карточке баннера. Логика вынесена из компонента, чтобы её
// можно было проверить тестами.
import type { FeaturedStatus } from '../api/types';
import type { TranslationKey } from '../i18n/locales/ru';

/** Известный статус 50/50 (вариант `known` из FeaturedStatus). */
export type KnownFeaturedStatus = Extract<FeaturedStatus, { state: 'known' }>;

/** Заголовок блока 50/50: главное, что нужно знать о следующем 5★. */
export function featuredHeadlineKey(status: KnownFeaturedStatus): TranslationKey {
  switch (status.chance_source) {
    case 'guaranteed':
      return 'featured.headline_guaranteed';
    case 'capturing_radiance':
      return 'featured.headline_radiance';
    case 'base':
      return 'featured.headline_fifty_fifty';
  }
}

/**
 * После скольких проигрышей подряд срабатывает Capturing Radiance
 * (серия + оставшиеся проигрыши). null — в игре нет этой механики.
 */
export function radianceThreshold(status: KnownFeaturedStatus): number | null {
  if (status.losses_until_radiance === null) return null;
  return status.losing_streak + status.losses_until_radiance;
}

/**
 * Показывать ли строку «Проигрышей до Capturing Radiance».
 * Не нужна, когда следующий 5★ и так гарантирован.
 */
export function showLossesUntilRadiance(status: KnownFeaturedStatus): boolean {
  return status.losses_until_radiance !== null && status.chance_source === 'base';
}
