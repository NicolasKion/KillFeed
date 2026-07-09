<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\DTO\OverlayOptions;
use App\Services\CharacterService;
use App\Services\KillmailFeedService;
use App\Services\ShipTypeService;
use App\Services\ZKillboardService;
use Illuminate\Container\Attributes\Config;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

final class HomeController extends Controller
{
    public function __construct(
        #[Config('killfeed.max_visible_killmails')] private readonly int $maxVisibleKillmails,
        #[Config('killfeed.max_killmail_age_hours')] private readonly int $maxKillmailAgeHours,
    ) {}

    public function index(ShipTypeService $shipTypeService)
    {
        return inertia('Home', [
            'max_visible_killmails' => $this->maxVisibleKillmails,
            'max_killmail_age_hours' => $this->maxKillmailAgeHours,
            'discord_invite_url' => config('killfeed.discord_invite_url'),
            'sim_ships' => $shipTypeService->demoShips(),
        ]);
    }

    public function demo(Request $request, ShipTypeService $shipTypeService)
    {
        $options = $this->options($request, default_summary: true);

        return inertia('Demo', [
            ...$options->toArray(),
            'sim_ships' => $shipTypeService->demoShips(),
        ]);
    }

    /**
     * @throws ConnectionException
     */
    public function preview(int $character, CharacterService $characterService): JsonResponse
    {
        $character = $characterService->resolve($character);

        return response()->json([
            'id' => $character->id,
            'name' => $character->name,
            'portrait' => "https://images.evetech.net/characters/{$character->id}/portrait?size=256",
        ]);
    }

    /**
     * @throws ConnectionException
     */
    public function show(
        int $character,
        Request $request,
        CharacterService $characterService,
        ZKillboardService $zKillboardService,
        KillmailFeedService $killmailFeedService,
    ) {
        $options = $this->options($request);

        $character = $characterService->resolve($character);

        $zKillboardService->queueRecentKillmails($character->id, $options->count);

        $character->listenNow();

        return inertia('Character', [
            ...$options->toArray(),
            ...$killmailFeedService->stats($character->id, $options)->toArray(),
            'character' => $character,
            'killmails' => $killmailFeedService->latest($character->id, $options),
        ]);
    }

    private function options(Request $request, bool $default_summary = false): OverlayOptions
    {
        return OverlayOptions::fromRequest($request, $this->maxVisibleKillmails, $this->maxKillmailAgeHours, $default_summary);
    }
}
