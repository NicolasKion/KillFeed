<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Models\Killmail;
use Illuminate\Console\Command;
use Illuminate\Container\Attributes\Config;

final class PurgeOldKillmails extends Command
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
    public function handle(#[Config('killfeed.max_killmail_age_hours')] int $maxKillmailAgeHours): int
    {
        $rows_deleted = Killmail::query()
            ->where('killmails.time', '<', now()->subHours($maxKillmailAgeHours))
            ->delete();

        $this->info(sprintf('Deleted %d killmails older than %d hours', $rows_deleted, $maxKillmailAgeHours));

        return self::SUCCESS;
    }
}
