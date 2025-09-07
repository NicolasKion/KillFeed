<?php

use App\Console\Commands\PurgeOldKillmails;
use Illuminate\Support\Facades\Schedule;

Schedule::command(PurgeOldKillmails::class)->daily();
