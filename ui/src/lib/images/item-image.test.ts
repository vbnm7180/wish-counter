// Тесты адресов картинок предметов (запуск: npm test).
import { describe, expect, it } from 'vitest';
import { buildItemImageUrl } from './item-image';

describe('buildItemImageUrl', () => {
  it('в Windows использует http://wcimg.localhost', () => {
    expect(buildItemImageUrl('genshin', 'hu_tao', true)).toBe('http://wcimg.localhost/genshin/hu_tao');
  });

  it('в других ОС использует схему wcimg://', () => {
    expect(buildItemImageUrl('genshin', 'hu_tao', false)).toBe('wcimg://localhost/genshin/hu_tao');
  });

  it('экранирует необычные символы', () => {
    expect(buildItemImageUrl('genshin', 'a/b', true)).toBe('http://wcimg.localhost/genshin/a%2Fb');
  });
});
