// The overlay page and its 10 second poll.
import { expect, test } from '@playwright/test';

import {
  assertSee,
  createCharacter,
  createKillmail,
  involve,
  psql,
  resetDatabase,
  stubEveImages,
  trackJavascriptErrors,
} from './helpers.ts';

test.beforeEach(async ({ page }) => {
  resetDatabase();
  await stubEveImages(page);
});

test('renders the killfeed overlay for a tracked character', async ({ page }) => {
  const noErrors = trackJavascriptErrors(page);
  const character = createCharacter({ name: 'Capsuleer Prime' });
  const killmail = createKillmail({ value: 1_000_000, metaGroup: 2 });
  involve(killmail, character, true);

  await page.goto(`/${character}`);

  await assertSee(page, '1M');
  await expect(page.locator('img[alt="Tech II"]')).toBeVisible();
  await expect(page).toHaveTitle('Capsuleer Prime - Killfeed');
  noErrors();
});

test('shows the summary panel with ISK totals when requested', async ({ page }) => {
  const noErrors = trackJavascriptErrors(page);
  const character = createCharacter();
  const kill = createKillmail({ value: 250_000_000 });
  involve(kill, character, true);
  const loss = createKillmail({ value: 50_000_000 });
  involve(loss, character, false);

  await page.goto(`/${character}?summary=1`);

  await assertSee(page, 'ISK destroyed');
  await assertSee(page, 'ISK lost');
  await assertSee(page, 'ISK efficiency');
  await assertSee(page, '83%');
  noErrors();
});

test('marks the character as being listened to', async ({ page }) => {
  const noErrors = trackJavascriptErrors(page);
  const character = createCharacter();
  expect(psql(`select listening_since from characters where id = ${character}`)).toBe('');

  await page.goto(`/${character}`);

  expect(psql(`select listening_since is not null from characters where id = ${character}`)).toBe(
    't',
  );
  noErrors();
});

test('polls every 10 seconds and shows a new killmail', async ({ page }) => {
  const noErrors = trackJavascriptErrors(page);
  const character = createCharacter();
  await page.goto(`/${character}`);
  await expect(page.getByText('N/A').first()).toBeVisible();

  const killmail = createKillmail({ value: 7_700_000 });
  involve(killmail, character, true);

  await expect(page.getByText('7.7M')).toBeVisible({ timeout: 15_000 });
  noErrors();
});

test('an unknown character is a 404', async ({ page }) => {
  const response = await page.goto('/1');
  expect(response?.status()).toBe(404);
  await assertSee(page, 'Not Found');
});
