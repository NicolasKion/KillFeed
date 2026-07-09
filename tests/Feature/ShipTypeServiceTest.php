<?php

declare(strict_types=1);

use App\Enums\MetaGroup;
use App\Services\ShipTypeService;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Http;

test('it resolves faction ships from the meta group attribute', function () {
    Http::fake([
        'https://esi.evetech.net/latest/universe/types/17738/' => Http::response([
            'type_id' => 17738,
            'name' => 'Machariel',
            'dogma_attributes' => [
                ['attribute_id' => 422, 'value' => 1.0],
                ['attribute_id' => 1692, 'value' => 4.0],
            ],
        ]),
    ]);

    expect(app(ShipTypeService::class)->metaGroupId(17738))->toBe(MetaGroup::Faction->value);
});

test('it resolves Tech II and Tech III hulls from the tech level attribute', function () {
    // ESI omits the meta group attribute (1692) on Tech II/III hulls.
    Http::fake([
        'https://esi.evetech.net/latest/universe/types/12044/' => Http::response([
            'type_id' => 12044,
            'name' => 'Enyo',
            'dogma_attributes' => [
                ['attribute_id' => 422, 'value' => 2.0],
            ],
        ]),
        'https://esi.evetech.net/latest/universe/types/29984/' => Http::response([
            'type_id' => 29984,
            'name' => 'Tengu',
            'dogma_attributes' => [
                ['attribute_id' => 422, 'value' => 3.0],
            ],
        ]),
    ]);

    $service = app(ShipTypeService::class);

    expect($service->metaGroupId(12044))->toBe(MetaGroup::TechII->value)
        ->and($service->metaGroupId(29984))->toBe(MetaGroup::TechIII->value);
});

test('it defaults to Tech I when no relevant attributes are present', function () {
    Http::fake([
        'https://esi.evetech.net/latest/universe/types/587/' => Http::response([
            'type_id' => 587,
            'name' => 'Rifter',
            'dogma_attributes' => [
                ['attribute_id' => 422, 'value' => 1.0],
                ['attribute_id' => 633, 'value' => 0.0],
            ],
        ]),
    ]);

    expect(app(ShipTypeService::class)->metaGroupId(587))->toBe(MetaGroup::TechI->value);
});

test('it caches the meta group per type id', function () {
    Http::fake([
        'https://esi.evetech.net/latest/universe/types/29990/' => Http::response([
            'type_id' => 29990,
            'name' => 'Loki',
            'dogma_attributes' => [
                ['attribute_id' => 422, 'value' => 3.0],
            ],
        ]),
    ]);

    $service = app(ShipTypeService::class);

    expect($service->metaGroupId(29990))->toBe(MetaGroup::TechIII->value)
        ->and($service->metaGroupId(29990))->toBe(MetaGroup::TechIII->value);

    Http::assertSentCount(1);
});

test('it falls back to Tech I without caching when ESI fails', function () {
    Http::fake([
        'https://esi.evetech.net/latest/universe/types/999999/' => Http::response(null, 500),
    ]);

    expect(app(ShipTypeService::class)->metaGroupId(999999))->toBe(MetaGroup::TechI->value)
        ->and(Cache::get('ship_type:999999:meta_group_id'))->toBeNull();
});
