<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\DTO\R2Z2Killmail;
use App\Enums\CacheKey;
use App\Jobs\ProcessKillmail;
use App\Models\Character;
use Illuminate\Console\Command;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;

use function dispatch;
use function usleep;

final class ListenForKills extends Command
{
    protected $signature = 'app:listen-for-kills';

    protected $description = 'Listen to R2Z2 firehose for new killmails';

    private bool $shouldKeepRunning = true;

    public function handle(): void
    {
        $this->trap([SIGTERM, SIGQUIT], function (): void {
            $this->shouldKeepRunning = false;
        });

        $baseUrl = config('killfeed.r2z2.base_url');
        $maxConsecutive404s = config('killfeed.r2z2.max_consecutive_404s');

        $sequenceId = $this->resolveStartSequence($baseUrl);
        if ($sequenceId === null) {
            Log::error('R2Z2: Failed to bootstrap sequence');
            $this->error('Failed to bootstrap sequence from R2Z2');

            return;
        }

        Log::info("R2Z2: Starting polling from sequence {$sequenceId}");
        $this->info("Starting R2Z2 polling from sequence {$sequenceId}");
        $consecutive404s = 0;

        while ($this->shouldKeepRunning) {
            if (Cache::pull(CacheKey::StopListening->value)) {
                Log::info('R2Z2: Received shutdown signal, stopping');
                $this->info('Received shutdown signal, stopping.');
                break;
            }

            $response = Http::get("{$baseUrl}/{$sequenceId}.json");

            if ($response->status() === 404) {
                $consecutive404s++;
                Log::debug("R2Z2: Sequence {$sequenceId} not available yet (404 #{$consecutive404s})");
                $this->warn("Sequence {$sequenceId} not available yet (404 #{$consecutive404s}), waiting 6s...");

                if ($consecutive404s >= $maxConsecutive404s) {
                    Log::warning("R2Z2: Hit {$consecutive404s} consecutive 404s, re-fetching sequence.json");
                    $this->warn("Hit {$consecutive404s} consecutive 404s, re-fetching sequence.json");
                    $newSequence = $this->fetchCurrentSequence($baseUrl);
                    if ($newSequence !== null) {
                        Log::info("R2Z2: Skipped ahead from {$sequenceId} to {$newSequence}");
                        $this->info("Skipped ahead to sequence {$newSequence}");
                        $sequenceId = $newSequence;
                        $consecutive404s = 0;

                        continue;
                    }
                }

                sleep(6);

                continue;
            }

            if ($response->failed()) {
                Log::error("R2Z2: Request failed with status {$response->status()} for sequence {$sequenceId}");
                $this->error("Sequence {$sequenceId} failed with status {$response->status()}, retrying in 6s...");
                sleep(6);

                continue;
            }

            $consecutive404s = 0;

            $r2z2 = R2Z2Killmail::fromArray($sequenceId, $response->json());
            $esi = $r2z2->toEsiKillmail();

            $context = [
                'killmail_id' => $r2z2->killmailId(),
                'sequence_id' => $r2z2->sequenceId(),
                'victim_character_id' => $esi->victim->character_id,
                'victim_ship_type_id' => $esi->victim->ship_type_id,
                'solar_system_id' => $esi->solar_system_id,
                'attacker_count' => count($esi->attackers),
                'killmail_time' => $esi->killmail_time,
            ];
            Log::info('R2Z2: Received killmail', $context);
            $this->info("Killmail #{$r2z2->killmailId()} | System {$esi->solar_system_id} | Victim {$esi->victim->character_id} | Ship {$esi->victim->ship_type_id} | Attackers: ".count($esi->attackers)." | {$esi->killmail_time}");

            $characters = Cache::remember(CacheKey::CharactersToMonitor->value, 30, fn (): Collection => $this->getCharacterIdsToMonitor());

            $this->dispatchIfMonitored($r2z2, $esi, $characters);

            Cache::put(CacheKey::R2Z2Sequence->value, $sequenceId);
            $sequenceId++;
            usleep(100_000);
        }
    }

    private function resolveStartSequence(string $baseUrl): ?int
    {
        $cached = Cache::get(CacheKey::R2Z2Sequence->value);

        if ($cached !== null) {
            $this->info("Resuming from cached sequence {$cached}");

            return (int) $cached + 1;
        }

        return $this->fetchCurrentSequence($baseUrl);
    }

    private function fetchCurrentSequence(string $baseUrl): ?int
    {
        $response = Http::get("{$baseUrl}/sequence.json");

        if ($response->failed()) {
            return null;
        }

        return $response->json('sequence');
    }

    /**
     * @param  Collection<int, int>  $characters
     */
    private function dispatchIfMonitored(R2Z2Killmail $r2z2, \App\DTO\EsiKillmail $esi, Collection $characters): void
    {
        if ($characters->isEmpty()) {
            return;
        }

        $involvedCharacterIds = collect($esi->attackers)
            ->pluck('character_id')
            ->push($esi->victim->character_id)
            ->whereNotNull();

        if ($involvedCharacterIds->intersect($characters)->isEmpty()) {
            return;
        }

        Log::info("R2Z2: Dispatching killmail {$r2z2->killmailId()} for processing (matched monitored character)");
        $this->comment("  -> Dispatching killmail #{$r2z2->killmailId()} (matched monitored character)");

        dispatch(new ProcessKillmail(
            killmail_id: $r2z2->killmailId(),
            hash: $r2z2->hash(),
            zKillKillmail: $r2z2->toZKillKillmail(),
            esiKillmail: $esi,
        ));
    }

    /**
     * @return Collection<int, int>
     */
    private function getCharacterIdsToMonitor(): Collection
    {
        return Character::query()->where('listening_since', '>', now()->subMinutes(5))->pluck('id');
    }
}
