<?php

namespace App\Jobs;

use App\DTO\EsiKillmail;
use App\DTO\zKillKillmail;
use App\Models\Character;
use App\Models\Killmail;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Http;

class ProcessKillmail implements ShouldQueue
{
    use Queueable;

    /**
     * Create a new job instance.
     */
    public function __construct(
        public int            $killmail_id,
        public string         $hash,
        public ?zKillKillmail $zKillKillmail = null,
    )
    {
        //
    }

    /**
     * Execute the job.
     */
    public function handle(): void
    {
        $zKillKillmail = $this->zKillKillmail;

        if ($zKillKillmail === null) {
            // Fetch the killmail from zkillboard
            $url = "https://zkillboard.com/api/killID/{$this->killmail_id}/";
            $response = Http::retry(5, 1000)->get($url);

            if ($response->failed()) {
                return;
            }

            $data = $response->json();

            $zKillKillmail = zKillKillmail::fromArray($data[0]);
        }

        // Fetch killmail from ESI

        $url = "https://esi.evetech.net/latest/killmails/{$this->killmail_id}/{$this->hash}/";
        $response = Http::retry(5, 1000)->get($url);

        if ($response->failed()) {
            return;
        }

        $data = $response->json();

        $esi_killmail = EsiKillmail::fromArray($data);

        DB::transaction(function () use ($esi_killmail, $zKillKillmail) {

            $attackers = collect($esi_killmail->attackers)->whereNotNull('character_id')->whereNotNull('corporation_id');
            $character_ids = $attackers->push($esi_killmail->victim)->map(fn($attacker) => $attacker->character_id)->unique();

            $character_ids->each(fn($character_id) => Character::updateOrCreate(['id' => $character_id], ['id' => $character_id]));

            $killmail = Killmail::updateOrCreate(
                ['id' => $this->killmail_id],
                [
                    'id' => $this->killmail_id,
                    'hash' => $this->hash,
                    'character_id' => $esi_killmail->victim->character_id,
                    'data' => $esi_killmail,
                    'time' => $esi_killmail->killmail_time,
                    'solarsystem_id' => $esi_killmail->solar_system_id,
                    'corporation_id' => $esi_killmail->victim->corporation_id,
                    'alliance_id' => $esi_killmail->victim->alliance_id,
                    'ship_type_id' => $esi_killmail->victim->ship_type_id,
                    'damage_taken' => $esi_killmail->victim->damage_taken,
                    'value' => $zKillKillmail->total_value,
                    'solo' => $zKillKillmail->solo,
                ]
            );

            $involvements = $attackers->map(fn($attacker) => [
                'character_id' => $attacker->character_id,
                'is_attacker' => true,
            ])->push([
                'character_id' => $esi_killmail->victim->character_id,
            ]);

            $involvements->each(fn($involvement) => $killmail->killmailInvolvements()->updateOrCreate($involvement));
        }, 5);
    }
}
