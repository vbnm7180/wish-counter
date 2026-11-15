// Подстановка параметров в строки перевода: «счётчик {current} / {hard}».

/** Значения параметров строки перевода. */
export type TranslationParams = Record<string, string | number>;

/** Шаблон параметра: имя из латинских букв, цифр и _ в фигурных скобках. */
const PARAMETER_PATTERN = /\{(\w+)\}/g;

/**
 * Заменяет {имя} значениями из params.
 * Неизвестные параметры остаются как есть — так опечатку видно в интерфейсе.
 */
export function interpolate(template: string, params?: TranslationParams): string {
  if (!params) return template;
  return template.replace(PARAMETER_PATTERN, (placeholder, name: string) =>
    name in params ? String(params[name]) : placeholder,
  );
}
