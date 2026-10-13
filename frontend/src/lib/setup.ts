// The setup page's URL building, apart from the page so vitest can pin it.

export type OverlaySettings = { count: number; hours: number; summary: boolean; meta: boolean };

export function safeId(_input: string): string {
  throw new Error('not implemented');
}

export function overlayQuery(_settings: OverlaySettings): string {
  throw new Error('not implemented');
}

export function overlayUrl(_origin: string, _id: string, _settings: OverlaySettings): string {
  throw new Error('not implemented');
}

export function clamp(_value: number, _max: number): number {
  throw new Error('not implemented');
}

export function chatCommandMessage(_origin: string): string {
  throw new Error('not implemented');
}
