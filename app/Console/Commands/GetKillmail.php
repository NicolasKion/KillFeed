<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Jobs\ProcessKillmail;
use Illuminate\Console\Command;

final class GetKillmail extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:get-killmail {killmail_id} {hash}';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Command description';

    /**
     * Execute the console command.
     */
    public function handle(): void
    {
        ProcessKillmail::dispatchSync(
            killmail_id: (int) $this->argument('killmail_id'),
            hash: (string) $this->argument('hash'),
        );
    }
}
