// Screenshots of the Laravel pages, the baselines of
// frontend/e2e/visual.spec.ts: the pages of frontend/e2e/visual-pages.ts
// at desktop and phone width, with tests/fixtures/dataset.json seeded,
// Math.random seeded and the EVE images stubbed, exactly as the spec
// shoots the new stack. The <head> of the pages marked `head` goes to
// tests/fixtures/golden/heads/ for frontend/e2e/seo.spec.ts.
//
//   scripts/laravel-reference/run.sh start
//   python3 -I scripts/laravel-reference/record.py seed
//   cd frontend && node ../scripts/laravel-reference/capture.mjs
import { mkdirSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';

import { VIEWPORTS, VISUAL_PAGES, readHead } from '../../frontend/e2e/visual-pages.ts';

// Playwright from the frontend's dependencies; nothing is installed here.
const { chromium } = createRequire(new URL('../../frontend/package.json', import.meta.url))(
  'playwright',
);

const BASE = 'http://127.0.0.1:8316';
const APP_URL = 'https://killfeed.killmail.space';
const OUT = new URL('../../frontend/e2e/__screenshots__/visual.spec.ts/', import.meta.url);
const HEADS = new URL('../../tests/fixtures/golden/heads/', import.meta.url);
mkdirSync(OUT, { recursive: true });
mkdirSync(HEADS, { recursive: true });

const PNG = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkaGD4DwAD/wH+0uK0eAAAAABJRU5ErkJggg==',
  'base64',
);

const browser = await chromium.launch();
for (const viewport of VIEWPORTS) {
  const context = await browser.newContext({
    viewport: { width: viewport.width, height: viewport.height },
    locale: 'en-US',
    timezoneId: 'UTC',
  });
  for (const visual of VISUAL_PAGES) {
    const page = await context.newPage();
    await page.addInitScript((initial) => {
      let state = initial;
      Math.random = () => {
        state = (state + 0x6d2b79f5) | 0;
        let t = Math.imul(state ^ (state >>> 15), 1 | state);
        t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
      };
    }, 42);
    await page.route('https://images.evetech.net/**', (route) =>
      route.fulfill({ status: 200, contentType: 'image/png', body: PNG }),
    );
    await page.goto(BASE + visual.path);
    await page.waitForLoadState('networkidle');
    await page.evaluate(() => document.fonts.ready);
    await page.waitForTimeout(1500);
    const file = new URL(`${visual.name}-${viewport.device}.png`, OUT);
    await page.screenshot({ path: file.pathname, animations: 'disabled' });
    if (visual.head && viewport.device === 'desktop') {
      // The reference serves on its own address; production on APP_URL.
      const head = JSON.parse(JSON.stringify(await page.evaluate(readHead)).replaceAll(BASE, APP_URL));
      writeFileSync(new URL(`${visual.name}.json`, HEADS), `${JSON.stringify(head, null, 2)}\n`);
    }
    await page.close();
    console.log(`captured ${visual.name} (${viewport.device})`);
  }
  await context.close();
}
await browser.close();
