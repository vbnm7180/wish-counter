// Тесты подстановки параметров в строки перевода (запуск: npm test).
import { describe, expect, it } from 'vitest';
import { interpolate } from './interpolate';

describe('interpolate', () => {
  it('подставляет параметры', () => {
    expect(interpolate('счётчик {current} / {hard}', { current: 5, hard: 90 })).toBe('счётчик 5 / 90');
  });

  it('возвращает строку без изменений, если параметров нет', () => {
    expect(interpolate('После {count} проигрышей')).toBe('После {count} проигрышей');
  });

  it('оставляет неизвестные параметры как есть', () => {
    expect(interpolate('{known} и {missing}', { known: 'да' })).toBe('да и {missing}');
  });
});
