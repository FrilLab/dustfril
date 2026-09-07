import { describe, expect, it } from 'vitest';
import { formatSignedBytes } from './format';

describe('formatSignedBytes', () => {
  it('formats positive, negative, and zero raw byte deltas', () => {
    expect(formatSignedBytes(2048)).toBe('+2,048 bytes');
    expect(formatSignedBytes(-4096)).toBe('-4,096 bytes');
    expect(formatSignedBytes(0)).toBe('0 bytes');
  });
});
