<?php

namespace App\Http\Controllers;

use App\DTO\CharacterData;
use App\DTO\KillmailData;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Fluent;

class HomeController extends Controller
{

    public function index()
    {
        return inertia('Home');
    }

    public function show(?int $character = null)
    {
        $character = $this->fetchCharacter($character);
        // Fetch the characters latest killmail hashes from zkillboard
        $killmails = $this->fetchKillmails($character->id);
        // Fetch the latest n killmails for the character from esi
        $killmails = $killmails
            ->take(5)
            ->map(fn($killmail) => $this->fetchKillmail(\fluent($killmail)));

        return inertia('Character', [
            'character' => $character,
            'killmails' => $killmails,
        ]);
    }

    private function fetchCharacter(?int $character): CharacterData
    {
        $cache_key = "character-$character";

        // Check if the character is already in the cache
        if (Cache::has($cache_key)) {
            return Cache::get($cache_key);
        }

        // Fetch the character from esi
        $url = "https://esi.evetech.net/latest/characters/$character/";

        $response = Http::get($url);

        $data = fluent($response->json());

        $character = new CharacterData(
            id: $character,
            name: $data->get('name'),
        );

        Cache::put($cache_key, $character, now()->addHour());

        return $character;
    }

    private function fetchKillmails(int $character): Collection
    {
        // Fetch the characters latest killmail hashes from zkillboard
        $url = "https://zkillboard.com/api/characterID/$character/";
        $response = Http::get($url);

        $killmails = $response->json();

        return collect($killmails);
    }

    private function fetchKillmail(Fluent $killmail): KillmailData
    {
        $cache_key = "killmail-{$killmail->killmail_id}";

        // Check if the killmail is already in the cache
        if (Cache::has($cache_key)) {
            return Cache::get($cache_key);
        }

        // Fetch the latest n killmails for the character from esi
        $id = $killmail->killmail_id;
        $hash = $killmail->get('zkb.hash');

        $url = "https://esi.evetech.net/latest/killmails/$id/$hash/";

        $response = Http::get($url);

        $data = fluent($response->json());

        $killmail = new KillmailData(
            id: $killmail->killmail_id,
            hash: $killmail->get('zkb.hash'),
            character_id: $data->get('victim.character_id'),
            ship_type_id: $data->get('victim.ship_type_id'),
            solar_system_id: $data->get('solar_system_id'),
            corporation_id: $data->get('victim.corporation_id'),
            alliance_id: $data->get('victim.alliance_id'),
            damage_taken: $data->get('victim.damage_taken'),
            killmail_time: $data->get('killmail_time'),
            attackers: $data->get('attackers') ?? [],
            items: $data->get('victim.items') ?? [],
            value: $killmail->get('zkb.totalValue'),
            solo: $killmail->get('zkb.solo'),
        );

        Cache::put($cache_key, $killmail, now()->addHour());

        return $killmail;
    }
}
