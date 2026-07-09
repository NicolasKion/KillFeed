<?php

declare(strict_types=1);

namespace App\Services;

use App\DTO\FeedStats;
use App\DTO\OverlayOptions;
use App\Jobs\BackfillShipMetaGroup;
use App\Models\Killmail;
use Illuminate\Database\Eloquent\Collection;

final readonly class KillmailFeedService
{
    /**
     * The latest killmails involving a character within the time window.
     *
     * @return Collection<int, Killmail>
     */
    public function latest(int $character_id, OverlayOptions $options): Collection
    {
        $killmails = Killmail::query()
            ->whereHas('killmailInvolvements', fn ($q) => $q->where('character_id', $character_id))
            ->latest('id')
            ->with('killmailInvolvements')
            ->where('time', '>', now()->subHours($options->hours))
            ->take($options->count)
            ->get();

        // Killmails stored before meta groups were tracked heal in the background.
        $killmails
            ->whereNull('ship_meta_group_id')
            ->each(fn (Killmail $killmail) => BackfillShipMetaGroup::dispatch($killmail->id));

        return $killmails;
    }

    public function stats(int $character_id, OverlayOptions $options): FeedStats
    {
        /** @var object{killmails_count: int|string, kills_value: int|float|string, losses_value: int|float|string}|null $row */
        $row = Killmail::query()
            ->join('killmail_involvements', 'killmail_involvements.killmail_id', '=', 'killmails.id')
            ->where('killmail_involvements.character_id', $character_id)
            ->where('killmails.time', '>', now()->subHours($options->hours))
            ->selectRaw('COUNT(DISTINCT killmails.id) as killmails_count')
            ->selectRaw('COALESCE(SUM(CASE WHEN killmail_involvements.is_attacker THEN killmails.value ELSE 0 END), 0) as kills_value')
            ->selectRaw('COALESCE(SUM(CASE WHEN NOT killmail_involvements.is_attacker THEN killmails.value ELSE 0 END), 0) as losses_value')
            ->toBase()
            ->first();

        return new FeedStats(
            killmails_count: (int) ($row->killmails_count ?? 0),
            kills_value: (float) ($row->kills_value ?? 0),
            losses_value: (float) ($row->losses_value ?? 0),
        );
    }
}
