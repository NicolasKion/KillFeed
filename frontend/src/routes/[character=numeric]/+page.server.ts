import { loadPage } from '#lib/server/api.ts';

export const load = loadPage<Record<string, unknown>>;
