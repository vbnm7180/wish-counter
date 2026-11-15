// Русские строки интерфейса. Это «эталонный» словарь:
// по его ключам TypeScript проверяет полноту остальных языков.
export const ru = {
  'app.title': 'Wish Counter',

  'nav.home': 'Главная',
  'nav.games': 'Игры',
  'nav.settings': 'Настройки',

  'home.heading': 'Добро пожаловать в Wish Counter',
  'home.description': 'Трекер молитв для гача-игр. Выберите игру в меню слева.',

  'games.genshin.name': 'Genshin Impact',

  'game.banners': 'Баннеры',
  'game.total_pulls': 'Всего молитв',
  'game.pity_5': 'До гаранта 5★',
  'game.pity_4': 'До гаранта 4★',
  'game.view': 'Посмотреть',
  'game.placeholder': 'Подсчёт молитв появится на следующих этапах разработки.',

  'genshin.banner.character': 'Баннер события персонажа',
  'genshin.banner.weapon': 'Баннер события оружия',
  'genshin.banner.standard': 'Стандартный баннер',
  'genshin.banner.chronicled': 'Хроники',
  'genshin.banner.beginner': 'Баннер новичка',

  'settings.heading': 'Настройки',
  'settings.theme': 'Тема оформления',
  'settings.language': 'Язык',
  'settings.save_error': 'Не удалось сохранить настройки',

  'theme.dark': 'Тёмная',
  'theme.light': 'Светлая',

  'language.ru': 'Русский',
  'language.en': 'English',

  'error.load_failed': 'Не удалось загрузить данные',
} as const;

/** Ключ строки интерфейса. Опечатка в ключе — ошибка компиляции. */
export type TranslationKey = keyof typeof ru;
