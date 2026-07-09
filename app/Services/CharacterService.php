<?php

declare(strict_types=1);

namespace App\Services;

use App\Models\Character;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Support\Facades\Http;

final class CharacterService
{
    /**
     * Find a known character or resolve it from ESI, aborting with a 404
     * when the id does not exist.
     *
     * @throws ConnectionException
     */
    public function resolve(int $character_id): Character
    {
        $character = Character::query()->whereNotNull('name')->find($character_id);

        if ($character) {
            return $character;
        }

        $response = Http::get("https://esi.evetech.net/latest/characters/$character_id/");

        abort_if($response->failed(), 404);

        return Character::query()->updateOrCreate(
            ['id' => $character_id],
            [
                'id' => $character_id,
                'name' => $response->json('name'),
            ]
        );
    }
}
