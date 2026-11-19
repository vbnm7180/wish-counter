// Тесты адресов картинок (запуск: npm test).
import { describe, expect, it } from 'vitest';
import { buildImageUrl } from './item-image';

describe('buildImageUrl', () => {
  it('в Windows использует http://wcimg.localhost', () => {
    expect(buildImageUrl(['items', 'genshin', 'hu_tao'], true)).toBe('http://wcimg.localhost/items/genshin/hu_tao');
  });

  it('в других ОС использует схему wcimg://', () => {
    expect(buildImageUrl(['games', 'genshin', 'icon'], false)).toBe('wcimg://localhost/games/genshin/icon');
  });

  it('экранирует необычные символы внутри частей пути', () => {
    expect(buildImageUrl(['items', 'genshin', 'a/b'], true)).toBe('http://wcimg.localhost/items/genshin/a%2Fb');
  });
});
