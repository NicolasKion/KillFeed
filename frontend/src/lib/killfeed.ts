import factionIcon from '#lib/assets/meta/faction.png';
import officerIcon from '#lib/assets/meta/officer.png';
import storylineIcon from '#lib/assets/meta/storyline.png';
import t2Icon from '#lib/assets/meta/t2.png';
import t3Icon from '#lib/assets/meta/t3.png';

/** The site name in page titles (`<page> - Killfeed`). */
export const APP_NAME = 'Killfeed';

export type OverlayCharacter = {
  id: number;
  name: string | null;
};

export type SimShip = {
  type_id: number;
  meta_group_id: number;
};

export type OverlayInvolvement = {
  character_id: number | null;
};

export type OverlayKillmail = {
  id: number;
  character_id: number | null;
  ship_type_id: number;
  ship_meta_group_id?: number | null;
  value: number | null;
  /** 0 or 1 from the api (MySQL's tinyint), a boolean in the simulation. */
  solo: boolean | number;
  killmail_involvements: OverlayInvolvement[];
};

export const eveImageUrl = (
  category: 'types' | 'characters',
  id: number,
  variant: string,
  size: number,
): string => {
  return `https://images.evetech.net/${category}/${id}/${variant}?size=${size}`;
};

/**
 * In-game corner icons per EVE meta group that can appear on a ship.
 * Tech I is the baseline and intentionally has no badge, matching the
 * in-game icon treatment.
 */
export const META_BADGES: Record<number, { icon: string; label: string }> = {
  2: { icon: t2Icon, label: 'Tech II' },
  3: { icon: storylineIcon, label: 'Storyline' },
  4: { icon: factionIcon, label: 'Faction' },
  5: { icon: officerIcon, label: 'Officer' },
  14: { icon: t3Icon, label: 'Tech III' },
};
