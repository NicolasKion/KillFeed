<?php

namespace App\Console\Commands;

use App\DTO\EsiKillmailAttacker;
use App\DTO\RedisQKillmail;
use App\Models\Character;
use App\Services\KillmailService;
use Cache;
use Illuminate\Console\Command;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Str;

class ListenForKills extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:listen-for-kills';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Listen to RedisQ for new killmails';

    protected bool $should_keep_running = true;

    protected string $queue_id;

    function __construct()
    {
        parent::__construct();

        $this->queue_id = sprintf('killfeed_%s', Str::random(16));
    }

    /**
     * Execute the console command.
     */
    public function handle(KillmailService $killmailService): void
    {
        $this->trap([
            SIGTERM,
            SIGQUIT
        ], fn() => $this->should_keep_running = false);


        while ($this->should_keep_running) {
            $characters = Cache::remember('characters_to_monitor', 30, fn() => $this->getCharacterIdsToMonitor());

            if ($characters->isEmpty()) {
                $this->debug('No characters to monitor, sleeping for 30 seconds');
                sleep(30);
                continue;
            }

            $maybe_killmail = $this->fetchNextKillmail();
            if ($maybe_killmail === null) {
                $this->debug('No killmails to process, sleeping for 60 seconds');
                sleep(60);
                continue;
            }

            if (!$this->isKillmailOfInterest($characters, $maybe_killmail)) {
                $this->debug('Killmail not of interest, skipping');
                continue;
            }

            $this->debug('Processing killmail');
            $killmailService->store($maybe_killmail->esi, $maybe_killmail->zkb);
        }
    }

    private function getCharacterIdsToMonitor(): Collection
    {
        return Character::query()->where('listening_since', '>', now()->subMinutes(5))->pluck('id');
    }

    private function debug(string $message): void
    {
        if (config('app.debug')) {
            $this->info($message);
        }
    }

    private function fetchNextKillmail(): ?RedisQKillmail
    {
        $request = Http::get('https://zkillredisq.stream/listen.php', [
            'queueID' => $this->queue_id,
        ]);

        if ($request->failed()) {
            $this->error('Failed to fetch next killmail');
            return null;
        }

        if ($request->json('package') === null) {
            return null;
        }

        return RedisQKillmail::fromArray($request->json());
    }

    private function isKillmailOfInterest(Collection $characters, RedisQKillmail $killmail): bool
    {
        if ($characters->contains($killmail->esi->victim->character_id)) {
            return true;
        }

        return collect($killmail->esi->attackers)->contains(fn(EsiKillmailAttacker $attacker) => $characters->contains($attacker->character_id));
    }
}
