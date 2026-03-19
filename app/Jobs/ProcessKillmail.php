<?php

declare(strict_types=1);

namespace App\Jobs;

use App\DTO\EsiKillmail;
use App\DTO\zKillKillmail;
use App\Jobs\Middleware\NotDuringDowntime;
use App\Services\KillmailService;
use Illuminate\Contracts\Queue\ShouldBeUnique;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Illuminate\Support\Facades\Http;

final class ProcessKillmail implements ShouldBeUnique, ShouldQueue
{
    use Queueable;

    /**
     * Create a new job instance.
     */
    public function __construct(
        public int $killmail_id,
        public string $hash,
        public ?zKillKillmail $zKillKillmail = null,
        public ?EsiKillmail $esiKillmail = null,
    ) {
        //
    }

    /**
     * Execute the job.
     */
    public function handle(KillmailService $killmailService): void
    {
        $zKillKillmail = $this->zKillKillmail;
        $esiKillmail = $this->esiKillmail;

        if (! $zKillKillmail instanceof zKillKillmail) {
            return;
        }

        if (! $esiKillmail instanceof EsiKillmail) {
            $url = "https://esi.evetech.net/latest/killmails/{$this->killmail_id}/{$this->hash}/";
            $response = Http::retry(5, 1000)->get($url);

            if ($response->failed()) {
                return;
            }

            $esiKillmail = EsiKillmail::fromArray($response->json());
        }

        $killmailService->store($esiKillmail, $zKillKillmail);
    }

    public function uniqueId(): string
    {
        return (string) $this->killmail_id;
    }

    public function middleware(): array
    {
        return [new NotDuringDowntime()];
    }
}
