// Тесты применения фона и сообщений об ошибках (запуск: npm test).
import { describe, expect, it } from 'vitest';
import { applyBackground, backgroundCssValue, backgroundErrorKey } from './background';

describe('backgroundCssValue', () => {
  it('без картинки — none', () => {
    expect(backgroundCssValue(null)).toBe('none');
  });

  it('с картинкой — url("...")', () => {
    expect(backgroundCssValue('http://wcimg.localhost/backgrounds/abc')).toBe(
      'url("http://wcimg.localhost/backgrounds/abc")',
    );
  });
});

describe('applyBackground', () => {
  it('включает фон и полупрозрачные панели', () => {
    const root = document.createElement('div');

    applyBackground('http://wcimg.localhost/backgrounds/abc', root);

    expect(root.style.getPropertyValue('--app-background-image')).toBe(
      'url("http://wcimg.localhost/backgrounds/abc")',
    );
    expect(root.dataset.background).toBe('on');
  });

  it('выключает фон', () => {
    const root = document.createElement('div');
    applyBackground('http://wcimg.localhost/backgrounds/abc', root);

    applyBackground(null, root);

    expect(root.style.getPropertyValue('--app-background-image')).toBe('none');
    expect(root.dataset.background).toBeUndefined();
  });
});

describe('backgroundErrorKey', () => {
  it('известные коды — отдельные сообщения', () => {
    expect(backgroundErrorKey({ code: 'background_too_large', message: '' })).toBe('background.error_too_large');
    expect(backgroundErrorKey({ code: 'background_unsupported', message: '' })).toBe('background.error_unsupported');
    expect(backgroundErrorKey({ code: 'background_read_failed', message: '' })).toBe('background.error_read');
  });

  it('прочие ошибки — общее сообщение', () => {
    expect(backgroundErrorKey({ code: 'storage_io', message: '' })).toBe('background.error_failed');
    expect(backgroundErrorKey(new Error('сбой'))).toBe('background.error_failed');
  });
});
