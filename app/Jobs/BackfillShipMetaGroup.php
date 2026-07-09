<?php

declare(strict_types=1);

namespace App\Jobs;

use App\Models\Killmail;
use App\Services\ShipTypeService;
use Illuminate\Contracts\Queue\ShouldBeUnique;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;

/**
 * Heals killmails stored before ship meta groups were tracked.
 */
final class BackfillShipMetaGroup implements ShouldBeUnique, ShouldQueue
{
    use Queueable;

    public function __construct(public int $killmail_id)
    {
        //
    }

    public function handle(ShipTypeService $shipTypeService): void
    {
        $killmail = Killmail::find($this->killmail_id);

        if ($killmail === null || $killmail->ship_meta_group_id !== null) {
            return;
        }

        $killmail->update([
            'ship_meta_group_id' => $shipTypeService->metaGroupId($killmail->ship_type_id),
        ]);
    }

    public function uniqueId(): string
    {
        return (string) $this->killmail_id;
    }
}
