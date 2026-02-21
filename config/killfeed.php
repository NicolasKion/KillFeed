<?php

declare(strict_types=1);

return [
    'max_visible_killmails' => max(1, (int) env('KILLMAIL_MAX_VISIBLE', 50)),
    'max_killmail_age_hours' => max(1, (int) env('KILLMAIL_MAX_AGE_HOURS', 168)),
    'discord_invite_url' => env('KILLFEED_DISCORD_INVITE_URL'),
];
