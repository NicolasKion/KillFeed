// The setup page's URL building, apart from the page so vitest can pin it.

export type OverlaySettings = { count: number; hours: number; summary: boolean; meta: boolean };

/** The digits of what was typed or pasted. */
export function safeId(input: string): string {
  return input.replace(/\D/g, '');
}

/** The overlay's query string, in the order the setup page always wrote it. */
export function overlayQuery(settings: OverlaySettings): string {
  return new URLSearchParams({
    count: settings.count.toString(),
    summary: settings.summary ? '1' : '0',
    meta: settings.meta ? '1' : '0',
    hours: settings.hours.toString(),
  }).toString();
}

/** The OBS browser source URL, empty until there is an id. */
export function overlayUrl(origin: string, id: string, settings: OverlaySettings): string {
  if (id.length === 0) {
    return '';
  }
  return `${origin}/${id}?${overlayQuery(settings)}`;
}

/** A count or hours between 1 and the maximum. */
export function clamp(value: number, max: number): number {
  if (Number.isNaN(value)) {
    return 1;
  }
  return Math.max(1, Math.min(value, max));
}

export function chatCommandMessage(origin: string): string {
  return `Want a killfeed like this on your stream? Build your own at ${origin || 'this page'}`;
}
