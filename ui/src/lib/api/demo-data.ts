// Демо-данные для запуска интерфейса в браузере (`npm run dev`) без Rust.
// Показывают разные состояния карточек, чтобы было удобно верстать:
// гарант после проигрыша, неизвестный статус, пустой баннер.
// В настоящем приложении (Tauri) не используются.
import type { FeaturedStatus, GameStatistics, PityCounter, PoolSummary, ReferenceSummary, ServerRegion } from './types';

/** Выбранный в демо-режиме сервер (в браузере хранится до перезагрузки страницы). */
let demoServerRegion: ServerRegion | null = null;

/** Запоминает сервер, выбранный в демо-режиме. */
export function setDemoServerRegion(region: ServerRegion): void {
  demoServerRegion = region;
}

/** Статус «4★ с баннера» для демо-карточек. */
function fourStar(guaranteed: boolean, chance: number): FeaturedStatus {
  return {
    state: 'known',
    guaranteed,
    losing_streak: 0,
    losses_until_radiance: null,
    next_featured_chance: guaranteed ? 10000 : chance,
    chance_source: guaranteed ? 'guaranteed' : 'base',
  };
}

/** Стоимость одной молитвы в примогемах. */
const PRIMOGEMS_PER_WISH = 160;

/** Создаёт счётчик гаранта так же, как Rust (`PityCounter::new`). */
function counter(current: number, hardPity: number): PityCounter {
  return { current, hard_pity: hardPity, remaining: Math.max(hardPity - current, 1) };
}

/** Создаёт демо-пул с заданными значениями. */
function pool(
  poolId: string,
  totalWishes: number,
  fiveStar: PityCounter,
  fourStar: PityCounter,
  featured: PoolSummary['featured'],
  fourStarFeatured: PoolSummary['four_star_featured'] = null,
): PoolSummary {
  return {
    pool_id: poolId,
    total_wishes: totalWishes,
    primogems_spent: totalWishes * PRIMOGEMS_PER_WISH,
    five_star: fiveStar,
    four_star: fourStar,
    featured,
    four_star_featured: fourStarFeatured,
  };
}

/** Возвращает демо-статистику игры. */
export function demoGameStatistics(gameId: string): GameStatistics {
  const pools = [
    pool('character', 412, counter(63, 90), counter(4, 10), {
      state: 'known',
      guaranteed: false,
      losing_streak: 2,
      losses_until_radiance: 1,
      next_featured_chance: 5500,
      chance_source: 'base',
    }, fourStar(true, 5000)),
    pool('weapon', 96, counter(17, 80), counter(10, 10), null, fourStar(false, 7500)),
    pool('chronicled', 0, counter(0, 90), counter(0, 10), null),
    pool('standard', 238, counter(41, 90), counter(7, 10), null),
    pool('beginner', 20, counter(20, 90), counter(3, 10), null),
  ];
  return {
    game_id: gameId,
    server_region: demoServerRegion,
    total_wishes: pools.reduce((sum, item) => sum + item.total_wishes, 0),
    primogems_spent: pools.reduce((sum, item) => sum + item.primogems_spent, 0),
    pools,
  };
}

/** Возвращает демо-сводку о справочнике. */
export function demoReferenceSummary(gameId: string): ReferenceSummary {
  return {
    game_id: gameId,
    source_revision: '4b873a6decad0384665084c69e9927a9fa266555',
    generated_at: '2026-11-15 17:30:00',
    character_count: 128,
    weapon_count: 243,
    banner_count: 224,
    latest_banner: { short_name: 'Mitya', version: '7.2', start: '2026-11-04 06:00:00' },
    last_checked_at: '2026-11-17 08:00:00',
  };
}
