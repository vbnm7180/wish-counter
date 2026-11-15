// Типы данных, которыми интерфейс обменивается с Rust.
// Должны совпадать со структурами из crates/domain (serde).

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

/** Ошибка команды Rust (app-desktop error::CommandError). */
export interface CommandError {
  code: string;
  message: string;
}
