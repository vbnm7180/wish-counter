// Тесты компонента «Выбор фона» (запуск: npm test).
import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import type { BackgroundImage } from '../api/types';
import BackgroundPicker from './BackgroundPicker.svelte';

/** Две картинки для тестов. */
const BACKGROUNDS: BackgroundImage[] = [
  { id: 'aaa', format: 'png' },
  { id: 'bbb', format: 'webp' },
];

/** Отрисовывает компонент; возвращает функции-шпионы обработчиков. */
function renderPicker(selected: string | null, busy = false) {
  const handlers = { onSelect: vi.fn(), onAdd: vi.fn(), onRemove: vi.fn() };
  render(BackgroundPicker, {
    backgrounds: BACKGROUNDS,
    selected,
    urlOf: (id: string) => `http://wcimg.localhost/backgrounds/${id}`,
    busy,
    ...handlers,
  });
  return handlers;
}

describe('BackgroundPicker', () => {
  it('показывает миниатюры добавленных картинок', () => {
    renderPicker(null);

    const sources = Array.from(document.querySelectorAll('img')).map((img) => img.getAttribute('src'));
    expect(sources).toEqual(['http://wcimg.localhost/backgrounds/aaa', 'http://wcimg.localhost/backgrounds/bbb']);
  });

  it('отмечает выбранный фон', () => {
    renderPicker('bbb');

    const pressed = screen.getAllByRole('button', { pressed: true });
    expect(pressed).toHaveLength(1);
    expect(pressed[0].querySelector('img')?.getAttribute('src')).toContain('bbb');
  });

  it('без фона отмечена плитка «Без фона»', () => {
    renderPicker(null);

    expect(screen.getByRole('button', { pressed: true }).textContent?.trim()).toBe('Без фона');
  });

  it('сообщает о выборе, добавлении и удалении', async () => {
    const handlers = renderPicker('aaa');

    await fireEvent.click(screen.getByText('Без фона'));
    await fireEvent.click(document.querySelectorAll('img')[1].closest('button')!);
    await fireEvent.click(screen.getByText(/Добавить/));
    await fireEvent.click(screen.getAllByLabelText('Удалить фон')[0]);

    expect(handlers.onSelect.mock.calls).toEqual([[null], ['bbb']]);
    expect(handlers.onAdd).toHaveBeenCalledOnce();
    expect(handlers.onRemove).toHaveBeenCalledWith('aaa');
  });

  it('во время добавления кнопка «Добавить» неактивна', () => {
    renderPicker(null, true);

    expect((screen.getByText(/Добавить/) as HTMLButtonElement).disabled).toBe(true);
  });
});
