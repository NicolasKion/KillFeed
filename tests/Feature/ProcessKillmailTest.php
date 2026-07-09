<?php

declare(strict_types=1);

use App\DTO\EsiKillmail;
use App\DTO\zKillKillmail;
use App\Jobs\ProcessKillmail;
use App\Models\Killmail;
use Illuminate\Support\Facades\Http;

test('it stores a killmail when full data is provided without fetching it', function () {
    fakeEsiTypes(meta_group_id: 2);

    $esi = EsiKillmail::fromArray([
        'killmail_id' => 99999,
        'killmail_time' => '2024-01-15T12:00:00Z',
        'moon_id' => null,
        'solar_system_id' => 30002187,
        'victim' => [
            'character_id' => 93813310,
            'corporation_id' => 98000001,
            'alliance_id' => null,
            'faction_id' => null,
            'damage_taken' => 500,
            'ship_type_id' => 670,
            'items' => [],
        ],
        'attackers' => [
            [
                'character_id' => 95465499,
                'corporation_id' => 98000002,
                'alliance_id' => null,
                'faction_id' => null,
                'damage_done' => 500,
                'final_blow' => true,
                'security_status' => -2.5,
                'ship_type_id' => 587,
                'weapon_type_id' => 3170,
            ],
        ],
    ]);

    $zkb = zKillKillmail::fromArray([
        'killmail_id' => 99999,
        'zkb' => [
            'hash' => 'testhash123',
            'locationID' => 40009081,
            'fittedValue' => 5000.0,
            'totalValue' => 10000.0,
            'droppedValue' => 3000.0,
            'destroyedValue' => 7000.0,
            'points' => 10,
            'npc' => false,
            'solo' => true,
            'awox' => false,
            'labels' => [],
        ],
    ]);

    (new ProcessKillmail(99999, 'testhash123', $zkb, $esi))->handle(app(App\Services\KillmailService::class));

    Http::assertNotSent(fn ($request): bool => str_contains((string) $request->url(), '/killmails/'));

    $killmail = Killmail::find(99999);
    expect($killmail)->not->toBeNull()
        ->and($killmail->hash)->toBe('testhash123')
        ->and($killmail->ship_meta_group_id)->toBe(2);
});

test('it fetches ESI killmail when esiKillmail is null', function () {
    fakeEsiTypes();
    Http::fake([
        'https://esi.evetech.net/latest/killmails/88888/fallbackhash/' => Http::response([
            'killmail_id' => 88888,
            'killmail_time' => '2024-01-15T12:00:00Z',
            'moon_id' => null,
            'solar_system_id' => 30002187,
            'victim' => [
                'character_id' => 93813310,
                'corporation_id' => 98000001,
                'alliance_id' => null,
                'faction_id' => null,
                'damage_taken' => 300,
                'ship_type_id' => 670,
                'items' => [],
            ],
            'attackers' => [
                [
                    'character_id' => 95465499,
                    'corporation_id' => 98000002,
                    'alliance_id' => null,
                    'faction_id' => null,
                    'damage_done' => 300,
                    'final_blow' => true,
                    'security_status' => 1.0,
                    'ship_type_id' => 587,
                    'weapon_type_id' => 3170,
                ],
            ],
        ]),
    ]);

    $zkb = zKillKillmail::fromArray([
        'killmail_id' => 88888,
        'zkb' => [
            'hash' => 'fallbackhash',
            'locationID' => 40009081,
            'fittedValue' => 1000.0,
            'totalValue' => 2000.0,
            'droppedValue' => 500.0,
            'destroyedValue' => 1500.0,
            'points' => 5,
            'npc' => false,
            'solo' => false,
            'awox' => false,
            'labels' => [],
        ],
    ]);

    (new ProcessKillmail(88888, 'fallbackhash', $zkb))->handle(app(App\Services\KillmailService::class));

    Http::assertSentCount(2);

    $killmail = Killmail::find(88888);
    expect($killmail)->not->toBeNull()
        ->and($killmail->ship_meta_group_id)->toBe(1);
});

test('it returns early when zKillKillmail is null', function () {
    Http::fake();

    (new ProcessKillmail(77777, 'somehash'))->handle(app(App\Services\KillmailService::class));

    Http::assertNothingSent();
    expect(Killmail::find(77777))->toBeNull();
});
