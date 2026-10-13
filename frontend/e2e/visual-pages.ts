// The pages the visual comparison and the SEO heads cover, shared with
// scripts/laravel-reference/capture.mjs so both stacks shoot the same
// thing: tests/fixtures/dataset.json seeded, Math.random seeded, the EVE
// images stubbed.

export const VIEWPORTS = [
  { device: 'desktop', width: 1280, height: 800 },
  { device: 'phone', width: 390, height: 844 },
] as const;

export type VisualPage = { name: string; path: string; head?: boolean };

export const VISUAL_PAGES: VisualPage[] = [
  { name: 'home', path: '/', head: true },
  { name: 'demo', path: '/demo', head: true },
  { name: 'demo-summary', path: '/demo?summary=1&meta=0' },
  { name: 'character', path: '/2112625428', head: true },
  { name: 'character-summary', path: '/2112625428?summary=1&count=3&hours=24' },
];

/** What the page heads are compared by, read after hydration. Link
 * targets are compared resolved (SvelteKit's client makes the icon links
 * absolute; the served HTML has them as Laravel wrote them). */
export function readHead() {
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
    links: links.filter((link) => !link.includes('/build/') && !link.includes('/_app/')),
    lang: document.documentElement.lang,
  };
}
