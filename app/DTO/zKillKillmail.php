<?php

namespace App\DTO;

class zKillKillmail
{
    /**
     * Create a new class instance.
     */
    public function __construct(
        public int    $id,
        public string $hash,
        public int    $location_id,
        public float  $fitted_value,
        public float  $total_value,
        public int    $dropped_value,
        public int    $destroyed_value,
        public int    $points,
        public bool   $npc,
        public bool   $solo,
        public bool   $awox,
        public array  $labels,
    )
    {
        //
    }

    public static function fromArray(array $data): self
    {
        return new self(
            id: $data['killmail_id'],
            hash: $data['zkb']['hash'],
            location_id: $data['zkb']['locationID'],
            fitted_value: $data['zkb']['fittedValue'],
            total_value: $data['zkb']['totalValue'],
            dropped_value: $data['zkb']['droppedValue'],
            destroyed_value: $data['zkb']['destroyedValue'],
            points: $data['zkb']['points'],
            npc: $data['zkb']['npc'],
            solo: $data['zkb']['solo'],
            awox: $data['zkb']['awox'],
            labels: $data['zkb']['labels'],
        );
    }
}
