import { describe, expect, it } from 'vitest';

import { inertiaReload } from './inertia.ts';

describe('inertiaReload', () => {
  it('sends tabs left open on the Laravel site to a full reload', () => {
    const url = new URL('https://killfeed.killmail.space/2112625428?count=5');
    const response = inertiaReload(
      new Request(url, { headers: { 'X-Inertia': 'true', 'X-Inertia-Version': 'abc' } }),
      url,
    );
    expect(response?.status).toBe(409);
    expect(response?.headers.get('X-Inertia-Location')).toBe(
      'https://killfeed.killmail.space/2112625428?count=5',
    );
  });

  it('leaves ordinary requests alone', () => {
    const url = new URL('https://killfeed.killmail.space/2112625428');
    expect(inertiaReload(new Request(url), url)).toBeNull();
  });
});
