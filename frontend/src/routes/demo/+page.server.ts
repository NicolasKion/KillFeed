import type { SimShip } from '#lib/killfeed.ts';
import { loadPage } from '#lib/server/api.ts';

export type DemoProps = {
  count: number;
  hours: number;
  summary: boolean;
  meta: boolean;
  sim_ships: SimShip[];
};

export const load = loadPage<DemoProps>;
