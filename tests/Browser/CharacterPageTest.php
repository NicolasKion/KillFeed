<?php

declare(strict_types=1);

use App\Models\Character;
use App\Models\Killmail;

beforeEach(function () {
    fakeZkillboard();
});

it('renders the killfeed overlay for a tracked character', function () {
    $character = Character::factory()->create(['name' => 'Capsuleer Prime']);

    $killmail = Killmail::factory()->create(['value' => 1_000_000, 'ship_meta_group_id' => 2]);
    $killmail->killmailInvolvements()->create([
        'character_id' => $character->id,
        'is_attacker' => true,
    ]);

    $page = visit("/{$character->id}");

    $page->assertSee('1M')
        ->assertVisible('img[alt="Tech II"]')
        ->assertNoJavascriptErrors();
});

it('shows the summary panel with ISK totals when requested', function () {
    $character = Character::factory()->create();

    $kill = Killmail::factory()->create(['value' => 250_000_000]);
    $kill->killmailInvolvements()->create([
        'character_id' => $character->id,
        'is_attacker' => true,
    ]);

    $loss = Killmail::factory()->create(['value' => 50_000_000]);
    $loss->killmailInvolvements()->create([
        'character_id' => $character->id,
        'is_attacker' => false,
    ]);

    $page = visit("/{$character->id}?summary=1");

    $page->assertSee('ISK destroyed')
        ->assertSee('ISK lost')
        ->assertSee('ISK efficiency')
        ->assertNoJavascriptErrors();
});

it('marks the character as being listened to', function () {
    $character = Character::factory()->create();

    expect($character->listening_since)->toBeNull();

    visit("/{$character->id}")->assertNoJavascriptErrors();

    expect($character->refresh()->listening_since)->not->toBeNull();
});
