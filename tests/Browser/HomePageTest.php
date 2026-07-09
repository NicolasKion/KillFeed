<?php

declare(strict_types=1);

use Illuminate\Support\Facades\Http;

beforeEach(function () {
    fakeEsiTypes();
});

it('renders the overlay setup page', function () {
    $page = visit('/');

    $page->assertSee('Put your kills on stream.')
        ->assertSee('OBS browser source URL')
        ->assertNoJavascriptErrors();
});

it('runs the simulated feed until a character is entered', function () {
    $page = visit('/');

    $page->assertSee('SIM')
        ->assertSee('synthetic killmails')
        ->assertNoJavascriptErrors();
});

it('previews a character after entering an id', function () {
    Http::fake([
        'https://esi.evetech.net/latest/characters/2112625428/' => Http::response([
            'name' => 'Capsuleer Prime',
        ]),
    ]);

    $page = visit('/');

    $page->fill('character-id', '2112625428')
        ->assertSee('Capsuleer Prime')
        ->assertNoJavascriptErrors();
});
