<?php

namespace App\Jobs\Middleware;

use Closure;

class NotDuringDowntime
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
