// Тесты компонента «Подсказка i» (запуск: npm test).
import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import InfoTooltip from './InfoTooltip.svelte';

describe('InfoTooltip', () => {
  it('связывает кнопку с текстом подсказки', () => {
    render(InfoTooltip, { text: 'Сервер нужен для часового пояса' });

    const button = screen.getByRole('button');
    const tooltip = screen.getByRole('tooltip', { hidden: true });

    expect(tooltip.textContent).toBe('Сервер нужен для часового пояса');
    expect(button.getAttribute('aria-describedby')).toBe(tooltip.id);
  });
});
