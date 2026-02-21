<?php

declare(strict_types=1);

namespace App\DTO;

final class zKillKillmail
{
    /**
     * Create a new class instance.
     */
    public function __construct(
        public ?int $id,
        public string $hash,
        public int $location_id,
        public float $fitted_value,
        public float $total_value,
        public float $dropped_value,
        public float $destroyed_value,
        public int $points,
        public bool $npc,
        public bool $solo,
        public bool $awox,
        public array $labels,
    ) {
        //
    }

    public static function fromArray(array $data): self
    {
        return new self(
            id: $data['killmail_id'] ?? $data['killID'] ?? null,
            hash: $data['zkb']['hash'],
            location_id: $data['zkb']['locationID'] ?? 0,
            fitted_value: (float) $data['zkb']['fittedValue'],
            total_value: (float) $data['zkb']['totalValue'],
            dropped_value: (float) $data['zkb']['droppedValue'],
            destroyed_value: (float) $data['zkb']['destroyedValue'],
            points: $data['zkb']['points'],
            npc: $data['zkb']['npc'],
            solo: $data['zkb']['solo'],
            awox: $data['zkb']['awox'],
            labels: $data['zkb']['labels'],
        );
    }
}
