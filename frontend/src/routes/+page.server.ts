import type { SimShip } from '#lib/killfeed.ts';
import { loadPage } from '#lib/server/api.ts';

export type HomeProps = {
  max_visible_killmails: number;
  max_killmail_age_hours: number;
  discord_invite_url: string | null;
  sim_ships: SimShip[];
};

export const load = loadPage<HomeProps>;
