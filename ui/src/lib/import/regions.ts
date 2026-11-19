// Серверы игры для выпадающего списка (значения — как domain::region::ServerRegion).
import type { ServerRegion } from '../api/types';
import type { TranslationKey } from '../i18n/locales/ru';

/** Серверы в порядке отображения и ключи их названий. */
export const REGIONS: { id: ServerRegion; labelKey: TranslationKey }[] = [
  { id: 'america', labelKey: 'region.america' },
  { id: 'europe', labelKey: 'region.europe' },
  { id: 'asia', labelKey: 'region.asia' },
  { id: 'china', labelKey: 'region.china' },
];

/** Ключ названия сервера. */
export function regionLabelKey(region: ServerRegion): TranslationKey {
  return `region.${region}`;
}
