// The overlay component in a real browser: as many 128 pixel cards as the
// strip fits, N/A tiles for the empty slots, the +N tile, meta badges and
// the summary panel.
import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';

import type { OverlayKillmail } from '#lib/killfeed.ts';

import KillfeedOverlay from './killfeed-overlay.svelte';

const character = { id: 2112625428, name: 'Capsuleer Prime' };

function killmail(id: number, overrides: Partial<OverlayKillmail> = {}): OverlayKillmail {
  return {
    id,
    character_id: 93813310,
    ship_type_id: 587,
    ship_meta_group_id: 1,
    value: 1_000_000,
    solo: 0,
    killmail_involvements: [{ character_id: 2112625428 }, { character_id: 93813310 }],
    ...overrides,
  };
}

// The overlay has a 1 pixel border on both sides: 2 pixels more than
// the cards.
async function renderAt(width: number, props: Record<string, unknown>) {
  document.body.style.margin = '0';
  const screen = await render(KillfeedOverlay, {
    character,
    killmails: [],
    hours: 168,
    summary: false,
    showMeta: true,
    killsValue: 0,
    lossesValue: 0,
    killmailsCount: 0,
    ...props,
  });
  screen.container.style.width = `${width}px`;
  return screen;
}

describe('KillfeedOverlay', () => {
  it('fills the strip with cards and N/A tiles', async () => {
    const screen = await renderAt(642, {
      killmails: [killmail(1, { ship_meta_group_id: 2, solo: 1 })],
      killmailsCount: 1,
    });

    await expect.poll(() => screen.container.querySelectorAll('[data-slot]').length).toBe(5);
    await expect.element(screen.getByText('1M')).toBeVisible();
    await expect.element(screen.getByText('solo')).toBeVisible();
    await expect.element(screen.getByAltText('Tech II')).toBeVisible();
    expect(screen.getByText('N/A').elements()).toHaveLength(4);
  });

  it('shows the number of involved characters, a red dot for a loss', async () => {
    const screen = await renderAt(258, {
      killmails: [killmail(1), killmail(2, { character_id: 2112625428 })],
      killmailsCount: 2,
      showMeta: false,
    });

    await expect.poll(() => screen.container.querySelectorAll('.bg-red-500').length).toBe(1);
    expect(screen.container.querySelectorAll('.bg-green-500')).toHaveLength(1);
    expect(screen.getByText('2', { exact: true }).elements()).toHaveLength(2);
    expect(screen.container.querySelector('img[alt]')).toBeNull();
  });

  it('collapses what does not fit into the +N tile', async () => {
    const screen = await renderAt(386, {
      killmails: [killmail(1), killmail(2), killmail(3), killmail(4)],
      killmailsCount: 20,
      hours: 36,
    });

    await expect.element(screen.getByText('+ 17')).toBeVisible();
    await expect.element(screen.getByText('last 1.5 days')).toBeVisible();
  });

  it('adds the summary panel', async () => {
    const screen = await renderAt(1600, {
      summary: true,
      killsValue: 250_000_000,
      lossesValue: 0,
    });

    await expect.element(screen.getByText('ISK destroyed')).toBeVisible();
    await expect.element(screen.getByText('ISK lost')).toBeVisible();
    await expect.element(screen.getByText('ISK efficiency')).toBeVisible();
    await expect.element(screen.getByText('100%')).toBeVisible();
  });
});
