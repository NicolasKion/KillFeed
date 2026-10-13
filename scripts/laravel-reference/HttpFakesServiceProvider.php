<?php

// Copied into the reference checkout by run.sh (never part of main): every
// outgoing HTTP request is answered from storage/app/http-fakes.json,
// which record.py writes per scenario, and logged to
// storage/logs/http.jsonl. Unknown URLs answer 404.

declare(strict_types=1);

namespace App\Providers;

use Illuminate\Http\Client\Request;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\ServiceProvider;

final class HttpFakesServiceProvider extends ServiceProvider
{
    public function boot(): void
    {
        $path = storage_path('app/http-fakes.json');
        $fakes = is_file($path) ? json_decode((string) file_get_contents($path), true) : [];

        Http::fake(function (Request $request) use ($fakes) {
            file_put_contents(storage_path('logs/http.jsonl'), json_encode([
                'method' => $request->method(),
                'url' => $request->url(),
                'user_agent' => $request->header('User-Agent')[0] ?? null,
            ])."\n", FILE_APPEND);

            $fake = $fakes[$request->url()] ?? null;

            if ($fake === null) {
                return Http::response(null, 404);
            }

            return Http::response($fake['body'] ?? null, $fake['status'] ?? 200);
        });
    }
}
