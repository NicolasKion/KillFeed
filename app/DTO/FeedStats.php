<?php

declare(strict_types=1);

namespace App\DTO;

use Illuminate\Contracts\Support\Arrayable;

/**
 * @implements Arrayable<string, int|float>
 */
final readonly class FeedStats implements Arrayable
{
    public function __construct(
        public int $killmails_count,
        public float $kills_value,
        public float $losses_value,
    ) {}

    public function toArray(): array
    {
        return get_object_vars($this);
    }
}
