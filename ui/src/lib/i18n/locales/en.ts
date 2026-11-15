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
  'game.total_pulls': 'Total wishes',
  'game.pity_5': 'Until 5★ pity',
  'game.pity_4': 'Until 4★ pity',
  'game.view': 'View',
  'game.placeholder': 'Wish counting will arrive in the next development stages.',

  'genshin.banner.character': 'Character Event Wish',
  'genshin.banner.weapon': 'Weapon Event Wish',
  'genshin.banner.standard': 'Standard Wish',
  'genshin.banner.chronicled': 'Chronicled Wish',
  'genshin.banner.beginner': "Beginners' Wish",

  'settings.heading': 'Settings',
  'settings.theme': 'Theme',
  'settings.language': 'Language',
  'settings.save_error': 'Failed to save settings',

  'theme.dark': 'Dark',
  'theme.light': 'Light',

  'language.ru': 'Русский',
  'language.en': 'English',

  'error.load_failed': 'Failed to load data',
};
