// tests/Browser/DemoPageTest.php.
import { expect, test } from '@playwright/test';

import { assertSee, stubEveImages, trackJavascriptErrors } from './helpers.ts';

test.beforeEach(async ({ page }) => {
  await stubEveImages(page);
});

test('renders the simulated overlay', async ({ page }) => {
  const noErrors = trackJavascriptErrors(page);

  await page.goto('/demo');

  // The simulation seeds a few killmails on mount; the remaining slots
  // render as N/A placeholder tiles.
  await assertSee(page, 'N/A');
  await expect(page).toHaveTitle('Overlay demo - Killfeed');
  noErrors();
});

test('shows the summary panel when requested', async ({ page }) => {
  const noErrors = trackJavascriptErrors(page);

  await page.goto('/demo?summary=1');

  await assertSee(page, 'ISK destroyed');
  await assertSee(page, 'ISK lost');
  await assertSee(page, 'ISK efficiency');
  noErrors();
});
