// The <head> of every page against what the Laravel pages had after
// hydration (tests/fixtures/golden/heads, captured by
// scripts/laravel-reference/capture.mjs): title, description, OpenGraph
// and Twitter tags, icons, fonts, robots.
import { readFileSync } from 'node:fs';
import { expect, test } from '@playwright/test';

import { baseUrl, seedDataset, stubEveImages } from './helpers.ts';
import { VISUAL_PAGES, readHead } from './visual-pages.ts';

const APP_URL = 'https://killfeed.killmail.space';

test.beforeAll(() => {
  seedDataset();
});

for (const visual of VISUAL_PAGES.filter((page) => page.head)) {
  test(`the head of ${visual.name}`, async ({ page }) => {
    await stubEveImages(page);
    await page.goto(visual.path);
    await page.waitForLoadState('networkidle');

    const expected = JSON.parse(
      readFileSync(
        new URL(`../../tests/fixtures/golden/heads/${visual.name}.json`, import.meta.url),
        'utf8',
      ),
    );
    // The stack runs on its own origin; production on APP_URL.
    const head = JSON.parse(
      JSON.stringify(await page.evaluate(readHead)).replaceAll(baseUrl(), APP_URL),
    );
    expect(head).toEqual(expected);
  });
}
