export type OverlayCharacter = {
  id: number;
  name: string;
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
  solo: boolean | number;
  killmail_involvements: OverlayInvolvement[];
};

export const eveImageUrl = (
  _category: 'types' | 'characters',
  _id: number,
  _variant: string,
  _size: number,
): string => {
  throw new Error('not implemented');
};

export const META_BADGES: Record<number, { icon: string; label: string }> = {};
