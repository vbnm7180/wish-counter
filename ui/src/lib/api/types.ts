// Типы данных, которыми интерфейс обменивается с Rust.
// Должны совпадать со структурами из crates/domain и crates/application (serde).

/** Код языка интерфейса (domain::settings::Language). */
export type LanguageCode = 'ru' | 'en';

/** Пользовательские настройки (domain::settings::AppSettings). */
export interface AppSettings {
  /** Идентификатор темы: a-z, 0-9, '-'. */
  theme: string;
  language: LanguageCode;
  /** Выбранный фон (идентификатор картинки) или null — без фона. */
  background: string | null;
}

/** Формат картинки фона (domain::background::ImageFormat). */
export type ImageFormat = 'png' | 'jpeg' | 'webp';

/** Картинка фона, добавленная пользователем (domain::background::BackgroundImage). */
export interface BackgroundImage {
  id: string;
  format: ImageFormat;
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

/** Текст на языках интерфейса (domain::reference::LocalizedText). */
export interface LocalizedText {
  en: string;
  ru: string;
}

/** Предмет баннера (application FeaturedItem). */
export interface FeaturedItem {
  id: string;
  /** Название; null — предмета нет в справочнике. */
  name: LocalizedText | null;
}

/** Самый новый баннер справочника (application LatestBanner). */
export interface LatestBanner {
  short_name: string;
  version: string | null;
  /** Начало баннера, 'ГГГГ-ММ-ДД чч:мм:сс'. */
  start: string;
  /** Предметы 5★ с повышенным шансом. */
  featured: FeaturedItem[];
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

/** Формат файла истории для импорта (domain::import::ImportFormatId). */
export type ImportFormat = 'paimon-moe';

/** Итог импорта (application ImportSummary). */
export interface ImportSummary {
  /** Сколько молитв прочитано из файла (без пропущенных повторов). */
  total_in_file: number;
  /** Сколько добавлено. */
  added: number;
  /** Сколько уже было сохранено раньше. */
  already_present: number;
  /** Сколько повторно записанных молитв пропущено (ошибка в самом файле). */
  duplicates_skipped: number;
  /** Сервер из файла (сохранён), если был указан. */
  server_region: ServerRegion | null;
  /** UID из файла (сохранён), если был указан. */
  uid: string | null;
}

/** Профиль игрока в игре (domain::profile::GameProfile). */
export interface GameProfile {
  server_region: ServerRegion | null;
  /** UID игрока; null — неизвестен. */
  uid: string | null;
}

/** Результат экспорта (app-desktop ExportResult). */
export interface ExportResult {
  /** Путь к сохранённому файлу; null — пользователь отменил сохранение. */
  path: string | null;
  /** Сколько молитв в файле. */
  wish_count: number;
}

/** Исход 50/50 для молитвы (domain FiftyFiftyOutcome). */
export type FiftyFiftyOutcome = 'won' | 'lost' | 'guaranteed' | 'unknown';

/** Строка таблицы молитв (application WishRow). */
export interface WishRow {
  id: string;
  /** Время по серверу игры, 'ГГГГ-ММ-ДД чч:мм:сс'. */
  time: string;
  item_id: string;
  /** Название; null — предмета нет в справочнике. */
  name: LocalizedText | null;
  item_kind: 'character' | 'weapon';
  rarity: 3 | 4 | 5;
  /** Номер молитвы по гаранту (только 4★/5★). */
  pity: number | null;
  /** Исход 50/50 (только 4★/5★ на баннерах с этим правилом). */
  outcome: FiftyFiftyOutcome | null;
  /** Баннер в момент молитвы (например, 'Mitya'); null — неизвестен. */
  banner_name: string | null;
}

/** История пула баннеров (application PoolHistory). Строки — новые сверху. */
export interface PoolHistory {
  pool_id: string;
  server_region: ServerRegion | null;
  rows: WishRow[];
}
