import { afterEach, describe, expect, it, vi } from 'vitest';

import { SIMULATED_CHARACTER, createSimulatedKillfeed } from './simulated-killfeed.svelte.ts';

/** A seeded generator (mulberry32), so the simulation is repeatable. */
function seeded(seed: number): () => number {
  let state = seed;
  return () => {
    state = (state + 0x6d2b79f5) | 0;
    let t = Math.imul(state ^ (state >>> 15), 1 | state);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const ships = [
  { type_id: 587, meta_group_id: 1 },
  { type_id: 12044, meta_group_id: 2 },
];

afterEach(() => {
  vi.useRealTimers();
});

describe('createSimulatedKillfeed', () => {
  it('seeds killmails newest first and keeps the totals', () => {
    const feed = createSimulatedKillfeed({ ships, random: seeded(1) });
    feed.seed(5);

    expect(feed.killmails.map((killmail) => killmail.id)).toEqual([5, 4, 3, 2, 1]);
    expect(feed.killmailsCount).toBe(5);
    const losses = feed.killmails.filter(
      (killmail) => killmail.character_id === SIMULATED_CHARACTER.id,
    );
    const kills = feed.killmails.filter((killmail) => killmail.character_id === null);
    expect(feed.lossesValue).toBe(losses.reduce((sum, killmail) => sum + killmail.value!, 0));
    expect(feed.killsValue).toBe(kills.reduce((sum, killmail) => sum + killmail.value!, 0));
    for (const killmail of feed.killmails) {
      expect(ships.map((ship) => ship.type_id)).toContain(killmail.ship_type_id);
      expect(killmail.value).toBeGreaterThanOrEqual(Math.round(10 ** 6.2));
      expect(killmail.killmail_involvements.length).toBeGreaterThanOrEqual(1);
      if (killmail.solo) {
        expect(killmail.killmail_involvements).toHaveLength(1);
      }
    }
  });

  it('caps the list at maxKillmails but keeps counting', () => {
    const feed = createSimulatedKillfeed({ ships, maxKillmails: 3, random: seeded(2) });
    feed.seed(10);

    expect(feed.killmails).toHaveLength(3);
    expect(feed.killmails[0].id).toBe(10);
    expect(feed.killmailsCount).toBe(10);
  });

  it('is repeatable with the same seed', () => {
    const first = createSimulatedKillfeed({ ships, random: seeded(7) });
    const second = createSimulatedKillfeed({ ships, random: seeded(7) });
    first.seed(4);
    second.seed(4);

    expect(second.killmails).toEqual(first.killmails);
  });

  it('lands a new killmail every 2.5 to 6 seconds until stopped', () => {
    vi.useFakeTimers();
    const feed = createSimulatedKillfeed({ ships, random: seeded(3) });
    feed.start();

    vi.advanceTimersByTime(2499);
    expect(feed.killmailsCount).toBe(0);
    vi.advanceTimersByTime(6000 - 2499);
    expect(feed.killmailsCount).toBeGreaterThanOrEqual(1);
    vi.advanceTimersByTime(12_000);
    expect(feed.killmailsCount).toBeGreaterThanOrEqual(3);

    feed.stop();
    const count = feed.killmailsCount;
    vi.advanceTimersByTime(60_000);
    expect(feed.killmailsCount).toBe(count);
  });
});
