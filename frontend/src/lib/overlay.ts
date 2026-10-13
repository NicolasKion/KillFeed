// The overlay's arithmetic, apart from the component so vitest can pin it.

export const CARD_WIDTH = 128;

export function visibleSlots(_stripWidth: number): number {
  throw new Error('not implemented');
}

export function hiddenKillmails(_killmailsCount: number, _slots: number): number {
  throw new Error('not implemented');
}

export function timeWindowLabel(_hours: number): string {
  throw new Error('not implemented');
}

export function iskEfficiency(_kills: number, _losses: number): number {
  throw new Error('not implemented');
}

export function formatIsk(_value: number | null): string {
  throw new Error('not implemented');
}
