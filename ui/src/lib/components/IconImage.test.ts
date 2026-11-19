// Тесты компонента «Картинка с заглушкой» (запуск: npm test).
import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import IconImage from './IconImage.svelte';

describe('IconImage', () => {
  it('показывает картинку, если есть адрес', () => {
    render(IconImage, { src: 'http://wcimg.localhost/items/genshin/hu_tao', name: 'Ху Тао' });

    expect(screen.getByRole('img', { name: 'Ху Тао' })).toBeTruthy();
  });

  it('без адреса показывает первую букву названия', () => {
    render(IconImage, { src: null, name: 'Genshin Impact' });

    expect(screen.queryByRole('img')).toBeNull();
    expect(screen.getByText('G')).toBeTruthy();
  });

  it('при ошибке загрузки переключается на заглушку', async () => {
    render(IconImage, { src: 'http://wcimg.localhost/items/genshin/missing', name: 'Митя' });

    await fireEvent.error(screen.getByRole('img'));

    expect(screen.queryByRole('img')).toBeNull();
    expect(screen.getByText('М')).toBeTruthy();
  });
});
