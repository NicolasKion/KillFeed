// The overlay's arithmetic, apart from the component so vitest can pin it.

/** The width of one killmail card. */
export const CARD_WIDTH = 128;

const intl = new Intl.NumberFormat('en-US', {
  notation: 'compact',
  compactDisplay: 'short',
});

/** How many whole cards fit the strip; none before it is measured. */
export function visibleSlots(stripWidth: number): number {
  if (stripWidth <= 0) {
    return 0;
  }
  return Math.max(Math.floor(stripWidth / CARD_WIDTH), 0);
}

/** The killmails of the window beyond the visible slots (the `+N` tile). */
export function hiddenKillmails(killmailsCount: number, slots: number): number {
  return Math.max(killmailsCount - slots, 0);
}

export function timeWindowLabel(hours: number): string {
  if (hours > 24) {
    const days = (hours / 24).toFixed(1).replace(/\.0$/, '');
    return `last ${days} days`;
  }
  return `last ${hours} hours`;
}

/** The share of the ISK destroyed, 100 without losses. */
export function iskEfficiency(kills: number, losses: number): number {
  if (losses === 0) {
    return 100;
  }
  return (kills / (kills + losses)) * 100;
}

/** `1M`, `1.5B`: compact en-US; a killmail without a value is 0. */
export function formatIsk(value: number | null): string {
  return intl.format(value ?? 0);
}
