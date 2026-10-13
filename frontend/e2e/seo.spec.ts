// The <head> of every page after hydration: title, description,
// OpenGraph and Twitter tags, icons, fonts, robots.
import { expect, test } from '@playwright/test';

import { baseUrl, createCharacter, resetDatabase, stubEveImages } from './helpers.ts';

const APP_URL = 'https://killfeed.killmail.space';

const DESCRIPTION =
  "KillFeed turns your EVE Online character's live killmails into an OBS browser-source overlay: killmail cards with ship renders and ISK values, plus ISK destroyed, lost, and efficiency stats.";

const LINKS = [
  `icon ${APP_URL}/favicon.svg image/svg+xml`,
  `icon ${APP_URL}/favicon-32.png image/png 32x32`,
  `apple-touch-icon ${APP_URL}/apple-touch-icon.png`,
  'preconnect https://fonts.bunny.net/',
  'stylesheet https://fonts.bunny.net/css?family=figtree:400,500,600|chakra-petch:600,700|jetbrains-mono:400,600&display=swap',
];

function head(title: string, path: string, robots?: string) {
  return {
    title,
    meta: {
      viewport: 'width=device-width, initial-scale=1',
      description: DESCRIPTION,
      'og:type': 'website',
      'og:site_name': 'Killfeed',
      'og:title': 'KillFeed — EVE Online kill overlay for streamers',
      'og:description':
        'Put your kills on stream. A live killfeed overlay for EVE Online streamers, ready for OBS in three steps.',
      'og:image': `${APP_URL}/og.png`,
      'og:url': `${APP_URL}${path}`,
      'twitter:card': 'summary_large_image',
      ...(robots ? { robots } : {}),
    },
    links: LINKS,
    lang: 'en',
  };
}

/** The head as the tests compare it, with link targets resolved. */
function readHead() {
  const meta: Record<string, string | null> = {};
  for (const element of document.head.querySelectorAll('meta[name], meta[property]')) {
    meta[element.getAttribute('name') ?? element.getAttribute('property') ?? ''] =
      element.getAttribute('content');
  }
  const links = [...document.head.querySelectorAll('link')].map((link) =>
    [link.getAttribute('rel'), link.href, link.getAttribute('type'), link.getAttribute('sizes')]
      .filter((part) => part !== null)
      .join(' '),
  );
  return {
    title: document.title,
    meta,
    links: links.filter((link) => !link.includes('/_app/')),
    lang: document.documentElement.lang,
  };
}

test.beforeAll(() => {
  resetDatabase();
  createCharacter({ id: 2112625428, name: 'Capsuleer Prime' });
});

for (const [name, path, expected] of [
  ['home', '/', head('EVE Online Stream Overlay - Killfeed', '')],
  ['demo', '/demo', head('Overlay demo - Killfeed', '/demo', 'noindex')],
  ['character', '/2112625428', head('Capsuleer Prime - Killfeed', '/2112625428', 'noindex')],
] as const) {
  test(`the head of ${name}`, async ({ page }) => {
    await stubEveImages(page);
    await page.goto(path);
    await page.waitForLoadState('networkidle');

    // The stack runs on its own origin; production on APP_URL.
    const actual = JSON.parse(
      JSON.stringify(await page.evaluate(readHead)).replaceAll(baseUrl(), APP_URL),
    );
    expect(actual).toEqual(expected);
  });
}
