// Тесты подготовки статуса 50/50 к отображению (запуск: npm test).
import { describe, expect, it } from 'vitest';
import {
  featuredHeadlineKey,
  type KnownFeaturedStatus,
  radianceThreshold,
  showLossesUntilRadiance,
} from './featured';

/** Статус «свежий баннер»: 50/50, проигрышей нет, Capturing Radiance через 3. */
function status(overrides: Partial<KnownFeaturedStatus> = {}): KnownFeaturedStatus {
  return {
    state: 'known',
    guaranteed: false,
    losing_streak: 0,
    losses_until_radiance: 3,
    next_featured_chance: 5500,
    chance_source: 'base',
    ...overrides,
  };
}

describe('featuredHeadlineKey', () => {
  it('выбирает заголовок по причине шанса', () => {
    expect(featuredHeadlineKey(status())).toBe('featured.headline_fifty_fifty');
    expect(featuredHeadlineKey(status({ chance_source: 'guaranteed' }))).toBe('featured.headline_guaranteed');
    expect(featuredHeadlineKey(status({ chance_source: 'capturing_radiance' }))).toBe(
      'featured.headline_radiance',
    );
  });
});

describe('radianceThreshold', () => {
  it('складывает серию проигрышей и оставшиеся проигрыши', () => {
    expect(radianceThreshold(status({ losing_streak: 2, losses_until_radiance: 1 }))).toBe(3);
  });

  it('возвращает null, если механики нет', () => {
    expect(radianceThreshold(status({ losses_until_radiance: null }))).toBeNull();
  });
});

describe('showLossesUntilRadiance', () => {
  it('показывает остаток только при обычном 50/50', () => {
    expect(showLossesUntilRadiance(status())).toBe(true);
    expect(showLossesUntilRadiance(status({ chance_source: 'guaranteed' }))).toBe(false);
    expect(showLossesUntilRadiance(status({ losses_until_radiance: null }))).toBe(false);
  });
});
