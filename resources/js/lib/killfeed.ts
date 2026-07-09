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
    value: number;
    solo: boolean;
    killmail_involvements: OverlayInvolvement[];
};

export const eveImageUrl = (category: 'types' | 'characters', id: number, variant: string, size: number): string => {
    return `https://images.evetech.net/${category}/${id}/${variant}?size=${size}`;
};

import factionIcon from '../../images/meta/faction.png';
import officerIcon from '../../images/meta/officer.png';
import storylineIcon from '../../images/meta/storyline.png';
import t2Icon from '../../images/meta/t2.png';
import t3Icon from '../../images/meta/t3.png';

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
