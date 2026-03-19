<?php

declare(strict_types=1);

use App\DTO\R2Z2Killmail;

test('it parses an R2Z2 payload into killmail id, hash, and sequence', function () {
    $r2z2 = R2Z2Killmail::fromArray(100, r2z2Payload());

    expect($r2z2->killmailId())->toBe(12345)
        ->and($r2z2->hash())->toBe('abc123hash')
        ->and($r2z2->sequenceId())->toBe(100);
});

test('it converts to EsiKillmail', function () {
    $esi = R2Z2Killmail::fromArray(100, r2z2Payload())->toEsiKillmail();

    expect($esi->killmail_id)->toBe(12345)
        ->and($esi->solar_system_id)->toBe(30002187)
        ->and($esi->victim->character_id)->toBe(93813310)
        ->and($esi->victim->ship_type_id)->toBe(670)
        ->and($esi->attackers)->toHaveCount(1)
        ->and($esi->attackers[0]->character_id)->toBe(95465499);
});

test('it converts to zKillKillmail', function () {
    $zkb = R2Z2Killmail::fromArray(100, r2z2Payload())->toZKillKillmail();

    expect($zkb->hash)->toBe('abc123hash')
        ->and($zkb->total_value)->toBe(10000.50)
        ->and($zkb->npc)->toBeFalse()
        ->and($zkb->solo)->toBeTrue();
});

function r2z2Payload(): array
{
    return [
        'killmail_id' => 12345,
        'hash' => 'abc123hash',
        'esi' => [
            'killmail_id' => 12345,
            'killmail_time' => '2024-01-15T12:00:00Z',
            'solar_system_id' => 30002187,
            'victim' => [
                'character_id' => 93813310,
                'corporation_id' => 98000001,
                'damage_taken' => 500,
                'ship_type_id' => 670,
                'items' => [],
            ],
            'attackers' => [
                [
                    'character_id' => 95465499,
                    'corporation_id' => 98000002,
                    'damage_done' => 500,
                    'final_blow' => true,
                    'security_status' => -2.5,
                    'ship_type_id' => 587,
                    'weapon_type_id' => 3170,
                ],
            ],
        ],
        'zkb' => [
            'hash' => 'abc123hash',
            'locationID' => 40009081,
            'fittedValue' => 5000.25,
            'totalValue' => 10000.50,
            'droppedValue' => 3000.00,
            'destroyedValue' => 7000.50,
            'points' => 10,
            'npc' => false,
            'solo' => true,
            'awox' => false,
            'labels' => ['pvp'],
        ],
        'uploaded_at' => 1773950479,
        'sequence_id' => 100,
    ];
}
