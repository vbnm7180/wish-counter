// Типы данных, которыми интерфейс обменивается с Rust.
// Должны совпадать со структурами из crates/domain и crates/application (serde).

/** Код языка интерфейса (domain::settings::Language). */
export type LanguageCode = 'ru' | 'en';

/** Пользовательские настройки (domain::settings::AppSettings). */
export interface AppSettings {
  /** Идентификатор темы: a-z, 0-9, '-'. */
  theme: string;
  language: LanguageCode;
}

/** Описание игры (domain::game::GameInfo). */
export interface GameInfo {
  id: string;
  display_order: number;
}

/** Сервер игрока (domain::region::ServerRegion). */
export type ServerRegion = 'america' | 'europe' | 'asia' | 'china';

/** Ошибка команды Rust (app-desktop error::CommandError). */
export interface CommandError {
  code: string;
  message: string;
}

/** Счётчик гаранта одной редкости (domain::pity::statistics::PityCounter). */
export interface PityCounter {
  /** Молитв после последнего предмета этой редкости. */
  current: number;
  /** Жёсткий гарант (90, 80 или 10). */
  hard_pity: number;
  /** Сколько молитв осталось до гаранта (≥ 1). */
  remaining: number;
}

/** Причина шанса получить предмет с баннера (domain ChanceSource). */
export type ChanceSource = 'guaranteed' | 'capturing_radiance' | 'base';

/** Статус 50/50 (domain FeaturedStatus). */
export type FeaturedStatus =
  | { state: 'unknown' }
  | {
      state: 'known';
      /** Прошлое 50/50 проиграно — следующий 5★ гарантированно с баннера. */
      guaranteed: boolean;
      /** Сколько 50/50 проиграно подряд. */
      losing_streak: number;
      /** Сколько ещё проигрышей до гарантии Capturing Radiance (null — механики нет). */
      losses_until_radiance: number | null;
      /** Шанс, что следующий 5★ с баннера, в сотых долях процента (5500 = 55%). */
      next_featured_chance: number;
      chance_source: ChanceSource;
    };

/** Статистика пула баннеров (application PoolSummary + domain PoolStatistics). */
export interface PoolSummary {
  /** Идентификатор пула: character, weapon, chronicled, standard, beginner. */
  pool_id: string;
  primogems_spent: number;
  total_wishes: number;
  five_star: PityCounter;
  four_star: PityCounter;
  /** Статус 50/50 для 5★; null — правило к пулу не применяется. */
  featured: FeaturedStatus | null;
  /** Статус «4★ с баннера»; null — правило к пулу не применяется. */
  four_star_featured: FeaturedStatus | null;
}

/** Статистика игры (application GameStatistics). */
export interface GameStatistics {
  game_id: string;
  /** Сервер игрока; null — не выбран (исход 50/50 не определяется). */
  server_region: ServerRegion | null;
  total_wishes: number;
  primogems_spent: number;
  pools: PoolSummary[];
}

/** Самый новый баннер справочника (application LatestBanner). */
export interface LatestBanner {
  short_name: string;
  version: string | null;
  /** Начало баннера, 'ГГГГ-ММ-ДД чч:мм:сс'. */
  start: string;
}

/** Сводка о справочнике игры (application ReferenceSummary). */
export interface ReferenceSummary {
  game_id: string;
  /** Ревизия источника — коммит Paimon.moe. */
  source_revision: string;
  /** Когда создан снимок (UTC), 'ГГГГ-ММ-ДД чч:мм:сс'. */
  generated_at: string;
  character_count: number;
  weapon_count: number;
  banner_count: number;
  latest_banner: LatestBanner | null;
  /** Когда последний раз проверялись обновления (UTC); null — ни разу. */
  last_checked_at: string | null;
}

/** Результат обновления справочника (application UpdateOutcome). */
export type UpdateOutcome =
  | { status: 'updated'; revision: string }
  | { status: 'up_to_date' }
  | { status: 'not_due' }
  | { status: 'unsupported' };
