<?php

declare(strict_types=1);

namespace App\Services;

use Illuminate\Http\Client\ConnectionException;
use Illuminate\Http\Client\Response;
use Illuminate\Support\Facades\Http;

final class R2Z2Client
{
    /**
     * The firehose's current sequence number, or null when unavailable.
     *
     * @throws ConnectionException
     */
    public function currentSequence(): ?int
    {
        $response = Http::get($this->baseUrl().'/sequence.json');

        if ($response->failed()) {
            return null;
        }

        return $response->json('sequence');
    }

    /**
     * Fetch a single killmail payload by sequence number.
     *
     * @throws ConnectionException
     */
    public function fetch(int $sequence): Response
    {
        return Http::get($this->baseUrl()."/$sequence.json");
    }

    private function baseUrl(): string
    {
        return (string) config('killfeed.r2z2.base_url');
    }
}
