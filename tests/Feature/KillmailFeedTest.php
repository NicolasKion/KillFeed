<?php

declare(strict_types=1);

use App\Models\Character;
use App\Models\Killmail;

test('killmails stored before meta groups were tracked heal when served', function () {
    fakeZkillboard();
    fakeEsiTypes(tech_level: 2);

    $character = Character::factory()->create();

    $killmail = Killmail::factory()->create(['ship_meta_group_id' => null]);
    $killmail->killmailInvolvements()->create([
        'character_id' => $character->id,
        'is_attacker' => true,
    ]);

    $this->get("/{$character->id}")->assertOk();

    expect($killmail->refresh()->ship_meta_group_id)->toBe(2);
});
