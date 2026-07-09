<?php

declare(strict_types=1);

use Inertia\Testing\AssertableInertia;

beforeEach(function () {
    fakeEsiTypes();
});

test('demo page renders with defaults', function () {
    $this->get('/demo')
        ->assertOk()
        ->assertInertia(fn (AssertableInertia $page) => $page
            ->component('Demo')
            ->where('count', 50)
            ->where('hours', 168)
            ->where('summary', true)
            ->where('meta', true));
});

test('demo page clamps parameters to configured maximums', function () {
    $this->get('/demo?count=9999&hours=9999&summary=1&meta=0')
        ->assertOk()
        ->assertInertia(fn (AssertableInertia $page) => $page
            ->component('Demo')
            ->where('count', 50)
            ->where('hours', 168)
            ->where('summary', true)
            ->where('meta', false));
});
