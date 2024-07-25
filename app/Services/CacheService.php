<?php

namespace App\Services;

use App\DTO\KillmailData;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Cache;
use Psr\Container\ContainerExceptionInterface;
use Psr\Container\NotFoundExceptionInterface;

class CacheService
{
    const SUBSCRIBED_IDS = 'subscribed_ids';

    /**
     * @throws ContainerExceptionInterface
     * @throws NotFoundExceptionInterface
     */
    public function addSubscribedId(int $id): void
    {
        $ids = $this->getSubscribedIds();
        if (!in_array($id, $ids)) {
            $ids[] = $id;
            Cache::put(self::SUBSCRIBED_IDS, $ids);
        }
    }

    /**
     * @throws ContainerExceptionInterface
     * @throws NotFoundExceptionInterface
     */
    public function getSubscribedIds(): array
    {
        return Cache::get(self::SUBSCRIBED_IDS, fn() => []);
    }

    /**
     * @throws ContainerExceptionInterface
     * @throws NotFoundExceptionInterface
     */
    public function removeSubscribedId(int $id): void
    {
        $ids = $this->getSubscribedIds();
        if (in_array($id, $ids)) {
            $ids = array_diff($ids, [$id]);
            Cache::put(self::SUBSCRIBED_IDS, $ids);
        }
    }

    public function addCharacterKillmailIds(int $character_id, array $ids): void
    {
        $ids = collect($ids)->merge($this->getCharacterKillmailIds($character_id))->unique()->sortDesc()->values();
        Cache::put("killmails:character:$character_id", $ids);
    }

    public function getCharacterKillmailIds(int $character_id): Collection
    {
        return Cache::get("killmails:character:$character_id", fn() => collect());
    }

    public function hasKillmail(int $killmail_id): bool
    {
        return Cache::has("killmails:$killmail_id");
    }

    public function addKillmail(KillmailData $killmail): void
    {
        Cache::put("killmails:{$killmail->id}", $killmail);
    }

    public function getKillmail(int $killmail_id): KillmailData
    {
        return Cache::get("killmails:$killmail_id");
    }
}
