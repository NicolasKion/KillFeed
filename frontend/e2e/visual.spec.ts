// Screenshots of every page against the Laravel pages
// (scripts/laravel-reference/capture.mjs recorded the baselines), at
// desktop and phone width, within the 3% tolerance of
// playwright.config.ts. Baselines are host specific (fonts, rendering),
// so CI skips this suite.
import { expect, test } from '@playwright/test';

import { seedDataset, seedRandom, stubEveImages } from './helpers.ts';
import { VIEWPORTS, VISUAL_PAGES } from './visual-pages.ts';

test.skip(!!process.env.CI, 'baselines are host specific');

test.beforeAll(() => {
  seedDataset();
});

for (const viewport of VIEWPORTS) {
  for (const visual of VISUAL_PAGES) {
    test(`${visual.name} (${viewport.device})`, async ({ page }) => {
      await page.setViewportSize({ width: viewport.width, height: viewport.height });
      await seedRandom(page);
      await stubEveImages(page);
      await page.goto(visual.path);
      await page.waitForLoadState('networkidle');
      await page.evaluate(() => document.fonts.ready);
      // The cards slide in and the totals count up; then the page holds
      // still until the simulation's next killmail (2.5 seconds at least).
      await page.waitForTimeout(1500);
      await expect(page).toHaveScreenshot(`${visual.name}-${viewport.device}.png`);
    });
  }
}
