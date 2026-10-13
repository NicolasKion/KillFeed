import { describe, expect, it } from 'vitest';

import {
  CARD_WIDTH,
  formatIsk,
  hiddenKillmails,
  iskEfficiency,
  timeWindowLabel,
  visibleSlots,
} from './overlay.ts';

describe('visibleSlots', () => {
  it('fits whole 128 pixel cards into the strip', () => {
    expect(CARD_WIDTH).toBe(128);
    expect(visibleSlots(0)).toBe(0);
    expect(visibleSlots(-5)).toBe(0);
    expect(visibleSlots(127)).toBe(0);
    expect(visibleSlots(128)).toBe(1);
    expect(visibleSlots(1279)).toBe(9);
    expect(visibleSlots(1920)).toBe(15);
  });
});

describe('hiddenKillmails', () => {
  it('counts the killmails beyond the visible slots for the +N tile', () => {
    expect(hiddenKillmails(20, 8)).toBe(12);
    expect(hiddenKillmails(3, 8)).toBe(0);
    expect(hiddenKillmails(0, 0)).toBe(0);
  });
});

describe('timeWindowLabel', () => {
  it('names hours up to a day and days with one decimal beyond', () => {
    expect(timeWindowLabel(1)).toBe('last 1 hours');
    expect(timeWindowLabel(24)).toBe('last 24 hours');
    expect(timeWindowLabel(25)).toBe('last 1 days');
    expect(timeWindowLabel(36)).toBe('last 1.5 days');
    expect(timeWindowLabel(48)).toBe('last 2 days');
    expect(timeWindowLabel(168)).toBe('last 7 days');
    expect(timeWindowLabel(100)).toBe('last 4.2 days');
  });
});

describe('iskEfficiency', () => {
  it('is 100 without losses and the kill share otherwise', () => {
    expect(iskEfficiency(0, 0)).toBe(100);
    expect(iskEfficiency(250, 0)).toBe(100);
    expect(iskEfficiency(250_000_000, 50_000_000)).toBeCloseTo(83.333, 2);
    expect(iskEfficiency(0, 10)).toBe(0);
  });
});

describe('formatIsk', () => {
  it('writes compact en-US numbers', () => {
    expect(formatIsk(1_000_000)).toBe('1M');
    expect(formatIsk(250_000_000)).toBe('250M');
    expect(formatIsk(1_500_000_000)).toBe('1.5B');
    expect(formatIsk(12_345)).toBe('12K');
    expect(formatIsk(999)).toBe('999');
    expect(formatIsk(0)).toBe('0');
  });

  it('writes a killmail without a zKillboard value as 0', () => {
    expect(formatIsk(null)).toBe('0');
  });
});
