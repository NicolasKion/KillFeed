<?php

namespace App\Console\Commands;

use App\DTO\EsiKillmailAttacker;
use App\DTO\RedisQKillmail;
use App\Jobs\ProcessKillmail;
use App\Models\Character;
use App\Services\KillmailService;
use Illuminate\Console\Command;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Str;
use function dispatch;
use function sleep;

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

        $this->queue_id = $this->createQueueId();
    }

    private function createQueueId(): string
    {
        return sprintf('killfeed_%s', Str::random(16));
    }

    /**
     * Execute the console command.
     */
    public function handle( ): void
    {
        $this->trap([
            SIGTERM,
            SIGQUIT
        ], fn() => $this->should_keep_running = false);


        while ($this->should_keep_running) {
            $characters = Cache::remember('characters_to_monitor', 30, fn() => $this->getCharacterIdsToMonitor());

            if ($characters->isEmpty()) {
                $this->info('No characters to monitor, sleeping for 30 seconds');
                sleep(30);
                continue;
            }

            $maybe_killmail = $this->fetchNextKillmail();
            if ($maybe_killmail === null) {
                $this->info('No killmails to process, sleeping for 60 seconds');
                sleep(60);
                continue;
            }

            sleep(1); // Respect zKillboard rate limit

            dispatch(new ProcessKillmail($maybe_killmail->killmail_id,$maybe_killmail->zkb->hash, $maybe_killmail->zkb));
        }
    }

    /**
     * Get the IDs of characters that are currently being monitored.
     *
     * @return Collection<int, int>
     */
    private function getCharacterIdsToMonitor(): Collection
    {
        return Character::query()->where('listening_since', '>', now()->subMinutes(5))->pluck('id');
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
}
