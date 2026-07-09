<?php

declare(strict_types=1);

use Illuminate\Http\Client\Request;
use Illuminate\Support\Facades\Http;

test('outgoing requests identify the app and its maintainer', function () {
    Http::fake();

    Http::get('https://esi.evetech.net/latest/status/');

    Http::assertSent(function (Request $request): bool {
        $userAgent = $request->header('User-Agent')[0] ?? '';

        return str_contains($userAgent, 'KillFeed')
            && str_contains($userAgent, 'nicolaskion07@gmail.com')
            && str_contains($userAgent, 'Nicolas Kion');
    });
});
