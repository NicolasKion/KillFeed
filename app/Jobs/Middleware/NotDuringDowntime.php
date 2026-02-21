<?php

declare(strict_types=1);

namespace App\Jobs\Middleware;

use Closure;

final class NotDuringDowntime
{
    public function handle(object $job, Closure $next)
    {
        $is_downtime = now()->isBetween('11:00', '11:15');
        if ($is_downtime) {
            $job->release(now()->diffInSeconds('11:15'));
        }

        return $next($job);
    }
}
