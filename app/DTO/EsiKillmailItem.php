<?php

declare(strict_types=1);

namespace App\DTO;

final class EsiKillmailItem
{
    public function __construct(
        public int $flag,
        public int $item_type_id,
        public int $singleton,
        public ?int $quantity_destroyed = null,
        public ?int $quantity_dropped = null,
        public array $items = [] // ItemDTO[]
    ) {}

    public static function fromArray(array $data): self
    {
        return new self(
            flag: $data['flag'],
            item_type_id: $data['item_type_id'],
            singleton: $data['singleton'],
            quantity_destroyed: $data['quantity_destroyed'] ?? null,
            quantity_dropped: $data['quantity_dropped'] ?? null,
            items: array_map(self::fromArray(...), $data['items'] ?? [])
        );
    }
}
