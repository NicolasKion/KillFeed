<?php

declare(strict_types=1);

namespace App\Services;

use App\Enums\MetaGroup;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Http;

final class ShipTypeService
{
    /** Dogma attribute holding the type's meta group (faction, officer, storyline). */
    public const int META_GROUP_ATTRIBUTE_ID = 1692;

    /** Dogma attribute holding the tech level; ESI omits 1692 on Tech II/III hulls. */
    public const int TECH_LEVEL_ATTRIBUTE_ID = 422;

    /**
     * The demo feed's ship catalog with real meta groups from ESI, cached
     * as one entry so page renders cost a single cache read.
     *
     * @return array<int, array{type_id: int, meta_group_id: int}>
     */
    public function demoShips(): array
    {
        return Cache::remember('ship_type:demo_ships', now()->addDays(30), fn (): array => collect(config('killfeed.demo_ship_type_ids'))
            ->map(fn (int $type_id): array => [
                'type_id' => $type_id,
                'meta_group_id' => $this->metaGroupId($type_id),
            ])
            ->all());
    }

    /**
     * Resolve the meta group of a type, cached per type id since the
     * static data only changes with game expansions.
     */
    public function metaGroupId(int $type_id): int
    {
        $cache_key = "ship_type:$type_id:meta_group_id";

        $cached = Cache::get($cache_key);

        if ($cached !== null) {
            return (int) $cached;
        }

        try {
            $response = Http::retry(3, 500, throw: false)->get("https://esi.evetech.net/latest/universe/types/$type_id/");
        } catch (ConnectionException) {
            return MetaGroup::TechI->value;
        }

        // Failures are not cached so the next killmail retries the lookup.
        if ($response->failed()) {
            return MetaGroup::TechI->value;
        }

        $attributes = collect($response->json('dogma_attributes') ?? [])->pluck('value', 'attribute_id');

        $meta_group_id = (int) ($attributes[self::META_GROUP_ATTRIBUTE_ID] ?? match ((int) ($attributes[self::TECH_LEVEL_ATTRIBUTE_ID] ?? 1)) {
            2 => MetaGroup::TechII->value,
            3 => MetaGroup::TechIII->value,
            default => MetaGroup::TechI->value,
        });

        Cache::put($cache_key, $meta_group_id, now()->addDays(30));

        return $meta_group_id;
    }
}
