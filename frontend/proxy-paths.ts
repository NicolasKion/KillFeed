// The shared-origin routing contract, mirrored by deploy/Caddyfile: keep
// both in sync when a path changes.

// Paths Axum owns outright: `/api` (the page props and the health
// check), `/up` (the health page) and `/characters` (the character
// preview the setup page fetches). Everything else is a SvelteKit page or
// a static file.
export const axumPrefixes = ['/api', '/up', '/characters'];

export function isAxumRequest(_method: string, path: string): boolean {
  return axumPrefixes.some((prefix) => path === prefix || path.startsWith(`${prefix}/`));
}
