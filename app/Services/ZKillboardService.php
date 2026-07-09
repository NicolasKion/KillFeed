<?php

declare(strict_types=1);

namespace App\Services;

use App\DTO\zKillKillmail;
use App\Jobs\ProcessKillmail;
use App\Models\Killmail;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Http;

final class ZKillboardService
{
    /**
     * Fetch a character's latest killmail hashes from zKillboard and queue
     * any that are not stored yet. The zKillboard response is cached
     * briefly to stay within their rate limits.
     *
     * @throws ConnectionException
     */
    public function queueRecentKillmails(int $character_id, int $count): void
    {
        $cache_key = "character:$character_id:killmails";

        $killmails = Cache::get($cache_key);

        if (! $killmails) {
            $response = Http::retry(5)->get("https://zkillboard.com/api/characterID/$character_id/");

            if ($response->failed()) {
                return;
            }

            $killmails = $response->json();

            Cache::put($cache_key, $killmails, now()->addMinutes(5));
        }

        collect($killmails)
            ->map(fn (array $killmail): zKillKillmail => zKillKillmail::fromArray($killmail))
            ->take($count)
            ->each(fn (zKillKillmail $killmail) => ProcessKillmail::dispatchUnless(
                Killmail::where('id', $killmail->id)->exists(),
                $killmail->id,
                $killmail->hash,
                $killmail,
            ));
    }
}
