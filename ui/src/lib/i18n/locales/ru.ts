// Русские строки интерфейса. Это «эталонный» словарь:
// по его ключам TypeScript проверяет полноту остальных языков.
// Параметры в фигурных скобках ({count}) подставляет функция i18n.t().
export const ru = {
  'app.title': 'Wish Counter',

  'nav.home': 'Главная',
  'nav.games': 'Игры',
  'nav.settings': 'Настройки',

  'home.heading': 'Добро пожаловать в Wish Counter',
  'home.description': 'Трекер молитв для гача-игр. Выберите игру в меню слева.',

  'games.genshin.name': 'Genshin Impact',

  'game.banners': 'Баннеры',
  'game.summary_total': 'Всего молитв',
  'game.summary_primogems': 'Потрачено примогемов',
  'game.total_pulls': 'Всего молитв',
  'game.primogems': 'Примогемы',
  'game.pity_5': 'До гаранта 5★',
  'game.pity_4': 'До гаранта 4★',
  'game.pity_progress': 'счётчик {current} / {hard}',
  'game.view': 'Посмотреть',
  'game.placeholder': 'Импорт молитв появится на следующих этапах. Сейчас история пуста.',

  'genshin.banner.character': 'Баннер события персонажа',
  'genshin.banner.weapon': 'Баннер события оружия',
  'genshin.banner.standard': 'Стандартный баннер',
  'genshin.banner.chronicled': 'Хроники',
  'genshin.banner.beginner': 'Баннер новичка',

  'featured.headline_fifty_fifty': 'Следующий 5★: 50/50',
  'featured.headline_guaranteed': 'Следующий 5★ — гарантированно персонаж с баннера',
  'featured.headline_radiance': 'Capturing Radiance: следующее 50/50 гарантированно будет выиграно',
  'featured.chance': 'Шанс персонажа с баннера',
  'featured.losing_streak': 'Проиграно 50/50 подряд',
  'featured.until_radiance': 'Проигрышей до Capturing Radiance',
  'featured.radiance_rule': 'После {count} проигрышей 50/50 подряд следующее 50/50 выигрывается гарантированно.',
  'featured.unknown': 'Статус 50/50 появится после загрузки данных о баннерах.',

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
