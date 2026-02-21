<?php

declare(strict_types=1);

namespace App\DTO;

final readonly class EsiKillmailPosition
{
    public function __construct(
        public float $x,
        public float $y,
        public float $z
    ) {}

    public static function fromArray(array $data): self
    {
        return new self(
            x: $data['x'],
            y: $data['y'],
            z: $data['z']
        );
    }
}
