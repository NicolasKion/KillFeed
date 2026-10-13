import { describe, expect, it } from 'vitest';

import { chatCommandMessage, clamp, overlayQuery, overlayUrl, safeId } from './setup.ts';

describe('safeId', () => {
  it('keeps the digits of what was typed or pasted', () => {
    expect(safeId('2112625428')).toBe('2112625428');
    expect(safeId(' 21-12 62a5428 ')).toBe('2112625428');
    expect(safeId('zkillboard.com/character/95465499/')).toBe('95465499');
    expect(safeId('abc')).toBe('');
  });
});

describe('overlayQuery', () => {
  it('writes count, summary, meta and hours in that order', () => {
    expect(overlayQuery({ count: 50, hours: 168, summary: true, meta: true })).toBe(
      'count=50&summary=1&meta=1&hours=168',
    );
    expect(overlayQuery({ count: 3, hours: 24, summary: false, meta: false })).toBe(
      'count=3&summary=0&meta=0&hours=24',
    );
  });
});

describe('overlayUrl', () => {
  it('is empty until there is an id', () => {
    const settings = { count: 50, hours: 168, summary: true, meta: true };
    expect(overlayUrl('https://killfeed.killmail.space', '', settings)).toBe('');
    expect(overlayUrl('https://killfeed.killmail.space', '2112625428', settings)).toBe(
      'https://killfeed.killmail.space/2112625428?count=50&summary=1&meta=1&hours=168',
    );
  });
});

describe('clamp', () => {
  it('keeps a value between 1 and the maximum', () => {
    expect(clamp(0, 50)).toBe(1);
    expect(clamp(-3, 50)).toBe(1);
    expect(clamp(20, 50)).toBe(20);
    expect(clamp(9999, 50)).toBe(50);
    expect(clamp(Number.NaN, 50)).toBe(1);
  });
});

describe('chatCommandMessage', () => {
  it('points viewers at the setup page', () => {
    expect(chatCommandMessage('https://killfeed.killmail.space')).toBe(
      'Want a killfeed like this on your stream? Build your own at https://killfeed.killmail.space',
    );
    expect(chatCommandMessage('')).toBe(
      'Want a killfeed like this on your stream? Build your own at this page',
    );
  });
});
