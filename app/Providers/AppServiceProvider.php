<?php

declare(strict_types=1);

namespace App\Providers;

use Illuminate\Support\Facades\Http;
use Illuminate\Support\ServiceProvider;

final class AppServiceProvider extends ServiceProvider
{
    /**
     * Register any application services.
     */
    public function register(): void
    {
        //
    }

    /**
     * Bootstrap any application services.
     */
    public function boot(): void
    {
        Http::globalRequestMiddleware(fn ($request) => $request->withHeader(
            'User-Agent', 'KillFeed/1.0 (https://killfeed.killmail.space; nicolaskion07@gmail.com; EVE: Nicolas Kion)'
        ));
    }
}
