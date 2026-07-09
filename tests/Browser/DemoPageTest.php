<?php

declare(strict_types=1);

beforeEach(function () {
    fakeEsiTypes();
});

it('renders the simulated overlay', function () {
    $page = visit('/demo');

    // The simulation seeds a few killmails on mount; the remaining slots
    // render as N/A placeholder tiles.
    $page->assertSee('N/A')
        ->assertNoJavascriptErrors();
});

it('shows the summary panel when requested', function () {
    $page = visit('/demo?summary=1');

    $page->assertSee('ISK destroyed')
        ->assertSee('ISK lost')
        ->assertSee('ISK efficiency')
        ->assertNoJavascriptErrors();
});
