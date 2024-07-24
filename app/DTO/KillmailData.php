<?php

namespace App\DTO;

readonly class KillmailData
{
    /**
     * Create a new class instance.
     */
    public function __construct(
        public int    $id,
        public string $hash,
        public int    $character_id,
        public int    $ship_type_id,
        public int    $solar_system_id,
        public int    $corporation_id,
        public ?int   $alliance_id,
        public int    $damage_taken,
        public string $killmail_time,
        public array  $attackers,
        public array  $items,
        public float  $value,
        public bool   $solo,
    )
    {
        //
    }
}
