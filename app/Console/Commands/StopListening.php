<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Enums\CacheKey;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\Cache;

final class StopListening extends Command
{
    protected $signature = 'app:stop-listening';

    protected $description = 'Signal the listen-for-kills process to shut down gracefully';

    public function handle(): void
    {
        Cache::put(CacheKey::StopListening->value, true, now()->addMinutes(5));
        $this->info('Shutdown signal sent. The listener will stop after its current iteration.');
    }
}
