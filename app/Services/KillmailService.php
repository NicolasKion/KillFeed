<?php

declare(strict_types=1);

namespace App\Services;

use App\DTO\EsiKillmail;
use App\DTO\zKillKillmail;
use App\Models\Character;
use App\Models\Killmail;
use Illuminate\Support\Facades\DB;

final readonly class KillmailService
{
    /**
     * Create a new class instance.
     */
    public function __construct(private ShipTypeService $shipTypeService)
    {
        //
    }

    public function store(EsiKillmail $esi_killmail, zKillKillmail $zKillKillmail): void
    {
        DB::transaction(function () use ($esi_killmail, $zKillKillmail): void {

            $killmail_id = $zKillKillmail->id ?? $esi_killmail->killmail_id;
            $hash = $zKillKillmail->hash;

            $attackers = collect($esi_killmail->attackers)->whereNotNull('character_id')->whereNotNull('corporation_id');
            $character_ids = $attackers->pluck('character_id')->push($esi_killmail->victim->character_id)->unique()->whereNotNull();

            $character_ids->each(fn ($character_id) => Character::updateOrCreate(['id' => $character_id], ['id' => $character_id]));

            $killmail = Killmail::updateOrCreate(
                ['id' => $killmail_id],
                [
                    'id' => $killmail_id,
                    'hash' => $hash,
                    'character_id' => $esi_killmail->victim->character_id,
                    'data' => $esi_killmail,
                    'time' => $esi_killmail->killmail_time,
                    'solarsystem_id' => $esi_killmail->solar_system_id,
                    'corporation_id' => $esi_killmail->victim->corporation_id,
                    'alliance_id' => $esi_killmail->victim->alliance_id,
                    'ship_type_id' => $esi_killmail->victim->ship_type_id,
                    'ship_meta_group_id' => $this->shipTypeService->metaGroupId($esi_killmail->victim->ship_type_id),
                    'damage_taken' => $esi_killmail->victim->damage_taken,
                    'value' => $zKillKillmail->total_value,
                    'solo' => $zKillKillmail->solo,
                ]
            );

            $involvements = $attackers->map(fn ($attacker): array => [
                'character_id' => $attacker->character_id,
                'is_attacker' => true,
            ]);

            $involvements->each(fn ($involvement) => $killmail->killmailInvolvements()->updateOrCreate($involvement));

            if ($esi_killmail->victim->character_id !== null) {
                $killmail->killmailInvolvements()->updateOrCreate([
                    'character_id' => $esi_killmail->victim->character_id,
                    'is_attacker' => false,
                ]);
            }

        });
    }
}
