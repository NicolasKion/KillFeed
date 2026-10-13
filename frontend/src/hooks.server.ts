import type { HandleFetch } from '@sveltejs/kit/hooks';
import { AXUM_URL } from '$app/env/private';

// Server-side loads reach Axum directly; the browser goes through the
// shared-origin proxy instead.
export const handleFetch: HandleFetch = async ({ event, request, fetch }) => {
  const url = new URL(request.url);
  if (url.origin !== event.url.origin || !url.pathname.startsWith('/api')) {
    return fetch(request);
  }
  const proxied = new Request(new URL(url.pathname + url.search, AXUM_URL), request);
  proxied.headers.set('host', event.url.host);
  proxied.headers.set('x-forwarded-proto', event.url.protocol.replace(':', ''));
  return fetch(proxied);
};
