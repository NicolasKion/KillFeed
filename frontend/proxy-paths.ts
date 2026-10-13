// The shared-origin routing contract, mirrored by deploy/Caddyfile.dev and
// deploy/Caddyfile: keep all three in sync when a path changes.

// Paths Axum owns outright: `/api` (the page props and the health
// check), `/up` (Laravel's health page) and `/characters` (the character
// preview the setup page fetches). Everything else is a SvelteKit page or
// a static file. Requests carrying `X-Inertia` (overlays left open on the
// Laravel site) go to Axum as well, by header, in the Caddyfiles; the
// SvelteKit hook answers them the same way when one gets through.
export const axumPrefixes = ['/api', '/up', '/characters'];

export function isAxumRequest(_method: string, path: string): boolean {
  return axumPrefixes.some((prefix) => path === prefix || path.startsWith(`${prefix}/`));
}
