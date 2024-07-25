<?php

namespace App\DTO;

/**
 * @property-read EsiKillmailItem[] $items
 */
readonly class EsiKillmailVictim
{
    public function __construct(
        public int                  $damage_taken,
        public array                $items = [], // ItemDTO[]
        public ?int                 $alliance_id = null,
        public ?int                 $character_id = null,
        public ?int                 $corporation_id = null,
        public ?int                 $faction_id = null,
        public ?EsiKillmailPosition $position = null,
        public ?int                 $ship_type_id = null
    )
    {
    }

    public static function fromArray(array $data): self
    {
        return new self(
            damage_taken: $data['damage_taken'],
            items: array_map(fn(array $item) => EsiKillmailItem::fromArray($item), $data['items']),
            alliance_id: $data['alliance_id'] ?? null,
            character_id: $data['character_id'] ?? null,
            corporation_id: $data['corporation_id'] ?? null,
            faction_id: $data['faction_id'] ?? null,
            position: $data['position'] ? EsiKillmailPosition::fromArray($data['position']) : null,
            ship_type_id: $data['ship_type_id'] ?? null
        );
    }
}
