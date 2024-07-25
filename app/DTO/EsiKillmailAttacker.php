<?php

namespace App\DTO;

class EsiKillmailAttacker
{
    public function __construct(
        public ?int  $alliance_id,
        public ?int  $character_id,
        public ?int  $corporation_id,
        public int   $damage_done,
        public ?int  $faction_id,
        public bool  $final_blow,
        public float $security_status,
        public ?int  $ship_type_id,
        public ?int  $weapon_type_id
    )
    {
    }

    public static function fromArray(array $data): self
    {
        return new self(
            alliance_id: $data['alliance_id'] ?? null,
            character_id: $data['character_id'] ?? null,
            corporation_id: $data['corporation_id'] ?? null,
            damage_done: $data['damage_done'],
            faction_id: $data['faction_id'] ?? null,
            final_blow: $data['final_blow'],
            security_status: $data['security_status'],
            ship_type_id: $data['ship_type_id'] ?? null,
            weapon_type_id: $data['weapon_type_id'] ?? null
        );
    }
}
