<?php

declare(strict_types=1);

namespace App\DTO;

use Illuminate\Contracts\Support\Arrayable;
use Illuminate\Http\Request;

/**
 * @implements Arrayable<string, int|bool>
 */
final readonly class OverlayOptions implements Arrayable
{
    public function __construct(
        public int $count,
        public int $hours,
        public bool $summary,
        public bool $meta,
    ) {}

    public static function fromRequest(Request $request, int $max_count, int $max_hours, bool $default_summary = false): self
    {
        return new self(
            count: max(1, min($request->integer('count', $max_count), $max_count)),
            hours: max(1, min($request->integer('hours', $max_hours), $max_hours)),
            summary: $request->boolean('summary', $default_summary),
            meta: $request->boolean('meta', true),
        );
    }

    public function toArray(): array
    {
        return get_object_vars($this);
    }
}
