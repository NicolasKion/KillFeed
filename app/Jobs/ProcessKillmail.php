<?php

namespace App\Jobs;

use App\DTO\EsiKillmail;
use App\DTO\zKillKillmail;
use App\Jobs\Middleware\NotDuringDowntime;
use App\Services\KillmailService;
use Illuminate\Contracts\Queue\ShouldBeUnique;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Illuminate\Support\Facades\Http;

class ProcessKillmail implements ShouldQueue, ShouldBeUnique
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
    public function handle(KillmailService $killmailService): void
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

        $killmailService->store($esi_killmail, $zKillKillmail);
    }

    public function uniqueId(): string
    {
        return $this->killmail_id;
    }

    public function middleware(): array
    {
        return [new NotDuringDowntime()];
    }
}
