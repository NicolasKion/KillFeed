// The route contract of the whole stack (Caddy, SvelteKit, Axum), which
// the PHP suite never covered: what OBS overlays and old links rely on.
import { expect, test } from '@playwright/test';

import { createCharacter, resetDatabase } from './helpers.ts';

test.beforeAll(() => {
  resetDatabase();
  createCharacter({ id: 2112625428, name: 'Capsuleer Prime' });
});

test('the pages answer, trailing slashes and leading zeros included', async ({ request }) => {
  for (const path of [
    '/',
    '/demo',
    '/demo/',
    '/demo?count=5&summary=1',
    '/2112625428',
    '/2112625428/',
    '/02112625428?count=3',
  ]) {
    const response = await request.get(path, { maxRedirects: 0 });
    expect(response.status(), path).toBe(200);
  }
});

test('the api paths answer', async ({ request }) => {
  expect((await request.get('/up')).status()).toBe(200);
  expect((await request.get('/api/health')).status()).toBe(200);
  const preview = await request.get('/characters/2112625428/preview', {
    headers: { accept: 'application/json' },
  });
  expect(await preview.json()).toEqual({
    id: 2112625428,
    name: 'Capsuleer Prime',
    portrait: 'https://images.evetech.net/characters/2112625428/portrait?size=256',
  });
  expect((await request.get('/characters/abc/preview')).status()).toBe(404);
});

test('unknown and dropped paths are 404 pages', async ({ request }) => {
  for (const path of ['/nope', '/horizon', '/horizon/api/stats', '/sanctum/csrf-cookie', '/1']) {
    const response = await request.get(path);
    expect(response.status(), path).toBe(404);
    expect(await response.text(), path).toContain('Not Found');
  }
});

test('an Inertia poll reloads into the new page', async ({ request }) => {
  for (const path of ['/2112625428?count=5&summary=1', '/demo', '/']) {
    const response = await request.get(path, {
      headers: { 'X-Inertia': 'true', 'X-Inertia-Version': 'a7ee6c644d6c4bba14be2af9e65116a6' },
      maxRedirects: 0,
    });
    expect(response.status(), path).toBe(409);
    expect(
      new URL(response.headers()['x-inertia-location']).pathname +
        new URL(response.headers()['x-inertia-location']).search,
    ).toBe(path);
  }
});

test('the static files and the security headers', async ({ request }) => {
  for (const path of [
    '/favicon.svg',
    '/favicon.ico',
    '/favicon-32.png',
    '/apple-touch-icon.png',
    '/og.png',
    '/icons/2.png',
  ]) {
    expect((await request.get(path)).status(), path).toBe(200);
  }
  expect(await (await request.get('/robots.txt')).text()).toBe('User-agent: *\nDisallow:\n');
  const headers = (await request.get('/')).headers();
  expect(headers['x-frame-options']).toBe('SAMEORIGIN');
  expect(headers['x-content-type-options']).toBe('nosniff');
  expect(headers['x-xss-protection']).toBe('1; mode=block');
  expect(headers['set-cookie']).toBeUndefined();
});
