// Простая навигация между страницами без отдельной библиотеки-роутера.
// В десктопном приложении нет адресной строки, поэтому текущая страница —
// просто реактивная переменная.

/** Описание страницы. Новая страница = новый вариант в этом типе. */
export type Page = { kind: 'home' } | { kind: 'game'; gameId: string } | { kind: 'settings' };

class NavigationState {
  /** Текущая открытая страница. */
  current = $state<Page>({ kind: 'home' });

  /** Открывает указанную страницу. */
  goTo(page: Page): void {
    this.current = page;
  }

  /** Проверяет, открыта ли сейчас указанная страница (для подсветки в меню). */
  isActive(page: Page): boolean {
    const current = this.current;
    if (current.kind !== page.kind) return false;
    if (current.kind === 'game' && page.kind === 'game') return current.gameId === page.gameId;
    return true;
  }
}

/** Единственный экземпляр навигации. */
export const navigation = new NavigationState();
