// Тесты компонента «Выпадающий список» (запуск: npm test).
import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import Select from './Select.svelte';

/** Варианты для тестов. */
const OPTIONS = [
  { value: 'europe', label: 'Европа' },
  { value: 'asia', label: 'Азия' },
];

describe('Select', () => {
  it('показывает подсказку, пока ничего не выбрано', () => {
    render(Select, { options: OPTIONS, value: null, placeholder: 'Выберите сервер', label: 'Сервер', onChange: vi.fn() });

    const select = screen.getByLabelText('Сервер') as HTMLSelectElement;

    expect(select.value).toBe('');
    expect(screen.getByText('Выберите сервер')).toBeTruthy();
  });

  it('показывает выбранное значение без подсказки', () => {
    render(Select, { options: OPTIONS, value: 'asia', placeholder: 'Выберите сервер', label: 'Сервер', onChange: vi.fn() });

    expect((screen.getByLabelText('Сервер') as HTMLSelectElement).value).toBe('asia');
    expect(screen.queryByText('Выберите сервер')).toBeNull();
  });

  it('сообщает о выборе варианта', async () => {
    const onChange = vi.fn();
    render(Select, { options: OPTIONS, value: null, placeholder: '—', label: 'Сервер', onChange });

    await fireEvent.change(screen.getByLabelText('Сервер'), { target: { value: 'europe' } });

    expect(onChange).toHaveBeenCalledWith('europe');
  });
});
