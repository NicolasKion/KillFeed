<?php

declare(strict_types=1);

return [
    'max_visible_killmails' => max(1, (int) env('KILLMAIL_MAX_VISIBLE', 50)),
    'max_killmail_age_hours' => max(1, (int) env('KILLMAIL_MAX_AGE_HOURS', 168)),
    'discord_invite_url' => env('KILLFEED_DISCORD_INVITE_URL'),

    'r2z2' => [
        'base_url' => env('R2Z2_BASE_URL', 'https://r2z2.zkillboard.com/ephemeral'),
        'max_consecutive_404s' => (int) env('R2Z2_MAX_CONSECUTIVE_404S', 10),
    ],

    // Ships used by the simulated demo feed; frigate to titan, with
    // recognizable renders on images.evetech.net.
    'demo_ship_type_ids' => [587, 593, 602, 626, 638, 645, 12044, 17738, 24698, 29984, 29990, 34562, 671],
];
