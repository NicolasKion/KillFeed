import { describe, expect, it } from 'vitest';

import { META_BADGES, eveImageUrl } from './killfeed.ts';

describe('eveImageUrl', () => {
  it('links the image server', () => {
    expect(eveImageUrl('types', 587, 'render', 128)).toBe(
      'https://images.evetech.net/types/587/render?size=128',
    );
    expect(eveImageUrl('characters', 95569887, 'portrait', 64)).toBe(
      'https://images.evetech.net/characters/95569887/portrait?size=64',
    );
  });
});

describe('META_BADGES', () => {
  it('badges every meta group but Tech I', () => {
    expect(Object.keys(META_BADGES).map(Number)).toEqual([2, 3, 4, 5, 14]);
    expect(Object.values(META_BADGES).map((badge) => badge.label)).toEqual([
      'Tech II',
      'Storyline',
      'Faction',
      'Officer',
      'Tech III',
    ]);
    expect(META_BADGES[1]).toBeUndefined();
  });
});
