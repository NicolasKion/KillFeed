<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('killmails', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->foreignId('character_id')->nullable()->constrained();
            $table->string('hash');
            $table->json('data');
            $table->dateTime('time');
            $table->unsignedBigInteger('solarsystem_id')->index();
            $table->unsignedBigInteger('corporation_id')->nullable()->index();
            $table->unsignedBigInteger('alliance_id')->nullable()->index();
            $table->unsignedBigInteger('ship_type_id')->index();
            $table->unsignedBigInteger('damage_taken');
            $table->double('value')->nullable();
            $table->boolean('solo')->default(false);
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('killmails');
    }
};
