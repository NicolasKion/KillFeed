<?php

namespace App\Console\Commands;

use App\Models\Killmail;
use Illuminate\Console\Command;

class PurgeOldKillmails extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:purge-old-killmails';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Purge old killmails from the database';

    /**
     * Execute the console command.
     */
    public function handle():int
    {
        $rows_deleted = Killmail::query()
            ->where('killmails.time', '<', now()->subMonths(3))
            ->delete();

        $this->info(sprintf('Deleted %d old killmails', $rows_deleted));

        return self::SUCCESS;
    }
}
