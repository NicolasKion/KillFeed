<?php

namespace App\Http\Controllers;

use App\DTO\zKillKillmail;
use App\Jobs\ProcessKillmail;
use App\Models\Character;
use App\Models\Killmail;
use App\Services\CacheService;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Http;
use Psr\Container\ContainerExceptionInterface;
use Psr\Container\NotFoundExceptionInterface;

class HomeController extends Controller
{

    function __construct(public CacheService $cacheService)
    {
    }

    public function index()
    {
        return inertia('Home');
    }

    /**
     * @throws ContainerExceptionInterface
     * @throws NotFoundExceptionInterface
     */
    public function show(?int $character = null)
    {
        $count = request('count', 5);
        $hours = request('hours');

        // Get the current character
        $character = $this->fetchCharacter($character);
        // Fetch the characters latest killmail hashes from zkillboard
        $this->getKillmails($character->id, $count);

        $character->listenNow();

        return inertia('Character', [
            'character' => $character,
            'killmails' => Killmail::whereHas('killmailInvolvements', fn($q) => $q->where('character_id', $character->id))
                ->latest('id')
                ->with('killmailInvolvements')
                ->when($hours, fn($q) => $q->where('time', '>', now()->subHours($hours)))
                ->take($count)->get(),
            'count' => $count,
            'hours' => $hours,
        ]);
    }

    private function fetchCharacter(?int $character_id): Character
    {
        $character = Character::query()->whereNotNull('name')->find($character_id);

        if ($character) {
            return $character;
        }

        // Fetch the character from esi
        $url = "https://esi.evetech.net/latest/characters/$character_id/";

        $response = Http::get($url);

        abort_if($response->failed(), 404);

        $data = $response->json();

        return Character::query()->updateOrCreate(
            [
                'id' => $character_id,
            ], [
                'id' => $character_id,
                'name' => $data['name']
            ]
        );
    }


    private function getKillmails(int $character, int $count = 5): void
    {
        $cacheKey = "character:$character:killmails";

        $killmails = Cache::get($cacheKey);

        if (!$killmails) {
            $url = "https://zkillboard.com/api/characterID/$character/";
            $response = Http::retry(5)->get($url);

            if ($response->failed()) {
                return;
            }

            $killmails = $response->json();

            Cache::put($cacheKey, $killmails, now()->addMinutes(5));
        }

        $collection = collect($killmails)->map(fn(array $killmail) => zKillKillmail::fromArray($killmail))->take($count);

        $collection->each(fn(zKillKillmail $killmail) => ProcessKillmail::dispatchUnless(Killmail::where('id', $killmail->id)->exists(), $killmail->id, $killmail->hash, $killmail));
    }
}
