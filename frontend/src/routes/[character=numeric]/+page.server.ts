import type { OverlayCharacter, OverlayKillmail } from '#lib/killfeed.ts';
import { loadPage } from '#lib/server/api.ts';

export type CharacterProps = {
  count: number;
  hours: number;
  summary: boolean;
  meta: boolean;
  killmails_count: number;
  kills_value: number;
  losses_value: number;
  character: OverlayCharacter;
  killmails: OverlayKillmail[];
};

export const load = loadPage<CharacterProps>;
