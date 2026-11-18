// Тесты выбора сообщения о результате обновления справочника (запуск: npm test).
import { describe, expect, it } from 'vitest';
import { errorMessageKey, outcomeMessageKey } from './update-messages';

describe('outcomeMessageKey', () => {
  it('сообщает об обновлении', () => {
    expect(outcomeMessageKey({ status: 'updated', revision: 'abc' })).toBe('reference.updated');
  });

  it('сообщает, что данные уже актуальны', () => {
    expect(outcomeMessageKey({ status: 'up_to_date' })).toBe('reference.up_to_date');
    expect(outcomeMessageKey({ status: 'not_due' })).toBe('reference.up_to_date');
  });

  it('сообщает, что обновление не поддерживается', () => {
    expect(outcomeMessageKey({ status: 'unsupported' })).toBe('reference.unsupported');
  });
});

describe('errorMessageKey', () => {
  it('различает ошибку сети и испорченные данные', () => {
    expect(errorMessageKey({ code: 'reference_download_failed', message: '' })).toBe('reference.download_failed');
    expect(errorMessageKey({ code: 'reference_invalid', message: '' })).toBe('reference.invalid');
  });

  it('для остальных ошибок даёт общий текст', () => {
    expect(errorMessageKey({ code: 'storage_io', message: '' })).toBe('reference.update_failed');
    expect(errorMessageKey(new Error('boom'))).toBe('reference.update_failed');
    expect(errorMessageKey(null)).toBe('reference.update_failed');
  });
});
