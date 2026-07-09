<?php

declare(strict_types=1);

use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Http;
use Tests\TestCase;

pest()->extend(TestCase::class)
    ->use(RefreshDatabase::class)
    ->in('Feature', 'Browser');

/**
 * Fake the ESI type lookups used to resolve ship meta groups.
 */
function fakeEsiTypes(?int $meta_group_id = null, ?int $tech_level = null): void
{
    $attributes = collect([
        App\Services\ShipTypeService::META_GROUP_ATTRIBUTE_ID => $meta_group_id,
        App\Services\ShipTypeService::TECH_LEVEL_ATTRIBUTE_ID => $tech_level,
    ])
        ->reject(fn (?int $value): bool => $value === null)
        ->map(fn (int $value, int $attribute_id): array => ['attribute_id' => $attribute_id, 'value' => (float) $value])
        ->values();

    Http::fake([
        'https://esi.evetech.net/latest/universe/types/*' => Http::response([
            'dogma_attributes' => $attributes->all(),
        ]),
    ]);
}

/**
 * Fake zKillboard's character killmail listing.
 */
function fakeZkillboard(array $killmails = []): void
{
    Http::fake([
        'https://zkillboard.com/api/characterID/*' => Http::response($killmails),
    ]);
}
