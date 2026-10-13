// tests/Browser/HomePageTest.php, plus the URL the setup page builds.
import { expect, test } from '@playwright/test';

import {
  assertSee,
  createCharacter,
  resetDatabase,
  stubEveImages,
  trackJavascriptErrors,
} from './helpers.ts';

test.beforeEach(async ({ page }) => {
  await stubEveImages(page);
});

test('renders the overlay setup page', async ({ page }) => {
  const noErrors = trackJavascriptErrors(page);

  await page.goto('/');

  await assertSee(page, 'Put your kills on stream.');
  await assertSee(page, 'OBS browser source URL');
  await expect(page).toHaveTitle('EVE Online Stream Overlay - Killfeed');
  noErrors();
});

test('runs the simulated feed until a character is entered', async ({ page }) => {
  const noErrors = trackJavascriptErrors(page);

  await page.goto('/');

  await assertSee(page, 'SIM');
  await assertSee(page, 'synthetic killmails');
  noErrors();
});

test('previews a character after entering an id', async ({ page }) => {
  resetDatabase();
  createCharacter({ id: 2112625428, name: 'Capsuleer Prime' });
  const noErrors = trackJavascriptErrors(page);

  await page.goto('/');
  await page.locator('#character-id').fill('2112625428');

  await assertSee(page, 'Capsuleer Prime');
  await assertSee(page, 'LIVE');
  await expect(page.locator('code').first()).toContainText(
    '/2112625428?count=50&summary=1&meta=1&hours=168',
  );
  await expect(page.locator('iframe[title="KillFeed preview"]')).toHaveAttribute(
    'src',
    /\/2112625428\?count=50&summary=1&meta=1&hours=168$/,
  );
  noErrors();
});

test('tells an unknown character apart', async ({ page }) => {
  resetDatabase();

  await page.goto('/');
  await page.locator('#character-id').fill('1');

  await assertSee(
    page,
    'Character not found. Check the character ID from zKillboard and try again.',
  );
});

test('keeps the options within their limits', async ({ page }) => {
  resetDatabase();
  createCharacter({ id: 2112625428, name: 'Capsuleer Prime' });
  await page.goto('/');
  await page.locator('#character-id').fill('21x1262 5428');
  await expect(page.locator('#character-id')).toHaveValue('2112625428');

  await page.locator('#visible-killmails').fill('500');
  await page.locator('#killmail-age').fill('0');
  await page.getByText('Include summary panel').click();

  await expect(page.locator('code').first()).toContainText(
    '/2112625428?count=50&summary=0&meta=1&hours=1',
  );
});
