<?php

declare(strict_types=1);

use App\Models\Character;
use Illuminate\Support\Facades\Http;

test('character preview returns resolved character data', function () {
    Http::fake([
        'https://esi.evetech.net/latest/characters/2112625428/' => Http::response([
            'name' => 'Capsuleer Prime',
        ]),
    ]);

    $response = $this->get('/characters/2112625428/preview');

    $response->assertOk()->assertJson([
        'id' => 2112625428,
        'name' => 'Capsuleer Prime',
        'portrait' => 'https://images.evetech.net/characters/2112625428/portrait?size=256',
    ]);

    expect(Character::find(2112625428)?->name)->toBe('Capsuleer Prime');
});

test('character preview route only accepts numeric ids', function () {
    $this->get('/characters/not-a-number/preview')->assertNotFound();
});
