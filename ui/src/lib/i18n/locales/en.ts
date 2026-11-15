// Английские строки интерфейса.
// Тип Record<TranslationKey, string> гарантирует, что переведены все ключи.
import type { TranslationKey } from './ru';

export const en: Record<TranslationKey, string> = {
  'app.title': 'Wish Counter',

  'nav.home': 'Home',
  'nav.games': 'Games',
  'nav.settings': 'Settings',

  'home.heading': 'Welcome to Wish Counter',
  'home.description': 'A wish tracker for gacha games. Pick a game in the menu on the left.',

  'games.genshin.name': 'Genshin Impact',

  'game.banners': 'Banners',
  'game.summary_total': 'Total wishes',
  'game.summary_primogems': 'Primogems spent',
  'game.total_pulls': 'Total wishes',
  'game.primogems': 'Primogems',
  'game.pity_5': 'Until 5★ pity',
  'game.pity_4': 'Until 4★ pity',
  'game.pity_progress': 'pity {current} / {hard}',
  'game.view': 'View',
  'game.placeholder': 'Wish import will arrive in the next stages. The history is empty for now.',

  'genshin.banner.character': 'Character Event Wish',
  'genshin.banner.weapon': 'Weapon Event Wish',
  'genshin.banner.standard': 'Standard Wish',
  'genshin.banner.chronicled': 'Chronicled Wish',
  'genshin.banner.beginner': "Beginners' Wish",

  'featured.headline_fifty_fifty': 'Next 5★: 50/50',
  'featured.headline_guaranteed': 'Next 5★ is guaranteed to be the featured character',
  'featured.headline_radiance': 'Capturing Radiance: your next 50/50 is a guaranteed win',
  'featured.chance': 'Featured character chance',
  'featured.losing_streak': '50/50 losses in a row',
  'featured.until_radiance': 'Losses until Capturing Radiance',
  'featured.radiance_rule': 'After {count} lost 50/50s in a row, the next 50/50 is a guaranteed win.',
  'featured.unknown': 'No data about the banner of this 5★. Please update the reference data.',
  'featured.unknown_no_region': 'Choose your game server to determine 50/50 results.',
  'featured.four_guaranteed': 'Next 4★ is guaranteed to be featured',
  'featured.four_chance': 'Featured 4★ chance',

  'region.label': 'Server',
  'region.hint': 'Your server defines the time zone of the wish history — it is required to determine 50/50 results.',
  'region.save_error': 'Failed to save the server',
  'region.america': 'America',
  'region.europe': 'Europe',
  'region.asia': 'Asia, TW/HK/MO',
  'region.china': 'China',

  'settings.heading': 'Settings',
  'settings.theme': 'Theme',
  'settings.language': 'Language',
  'settings.save_error': 'Failed to save settings',
  'settings.reference': 'Reference data',

  'reference.counts': 'Characters: {characters} · Weapons: {weapons} · Banners: {banners}',
  'reference.latest_banner': 'Latest banner: {name} (version {version}), starts {start}',
  'reference.source': 'Paimon.moe data from {date}, revision {revision}',
  'reference.license': 'Data source: Paimon.moe (MIT license). Genshin Impact © HoYoverse.',

  'theme.dark': 'Dark',
  'theme.light': 'Light',

  'language.ru': 'Русский',
  'language.en': 'English',

  'error.load_failed': 'Failed to load data',
};
