import { describe, expect, it } from 'vitest';

import { isAxumRequest } from './proxy-paths.ts';

describe('isAxumRequest', () => {
  it('sends the api paths to the api', () => {
    for (const path of [
      '/api/pages/home',
      '/api/health',
      '/up',
      '/characters/2112625428/preview',
      '/characters/abc/preview',
    ]) {
      expect(isAxumRequest('GET', path), path).toBe(true);
    }
  });

  it('keeps the pages and static files in SvelteKit', () => {
    for (const path of ['/', '/demo', '/2112625428', '/og.png', '/robots.txt', '/horizon']) {
      expect(isAxumRequest('GET', path), path).toBe(false);
    }
  });
});
