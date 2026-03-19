<?php

declare(strict_types=1);

namespace App\Enums;

enum CacheKey: string
{
    case StopListening = 'killfeed:stop_listening';
    case R2Z2Sequence = 'r2z2_sequence';
    case CharactersToMonitor = 'characters_to_monitor';
}
