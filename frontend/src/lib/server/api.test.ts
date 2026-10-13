import { describe, expect, it } from 'vitest';

import { pagesPath } from './api.ts';

describe('pagesPath', () => {
  it('maps a page URL to its props endpoint, query included', () => {
    const at = (path: string) => pagesPath(new URL(`https://killfeed.killmail.space${path}`));
    expect(at('/')).toBe('/api/pages/home');
    expect(at('/demo?summary=1')).toBe('/api/pages/demo?summary=1');
    expect(at('/demo/')).toBe('/api/pages/demo');
    expect(at('/2112625428?count=5&hours=24')).toBe('/api/pages/2112625428?count=5&hours=24');
    expect(at('/2112625428/?count=1')).toBe('/api/pages/2112625428?count=1');
  });
});
