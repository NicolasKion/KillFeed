<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Enums\MetaGroup;
use App\Models\Killmail;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<Killmail>
 */
final class KillmailFactory extends Factory
{
    protected $model = Killmail::class;

    public function definition(): array
    {
        return [
            'id' => fake()->unique()->numberBetween(100_000_000, 200_000_000),
            'character_id' => null,
            'hash' => fake()->sha1(),
            'data' => [],
            'time' => now()->subMinutes(fake()->numberBetween(1, 60)),
            'solarsystem_id' => fake()->numberBetween(30_000_000, 31_000_000),
            'corporation_id' => fake()->numberBetween(98_000_000, 99_000_000),
            'alliance_id' => null,
            'ship_type_id' => fake()->numberBetween(500, 40_000),
            'ship_meta_group_id' => fake()->randomElement([MetaGroup::TechI, MetaGroup::TechII, MetaGroup::Faction, MetaGroup::TechIII])->value,
            'damage_taken' => fake()->numberBetween(100, 100_000),
            'value' => fake()->randomFloat(2, 10_000, 5_000_000_000),
            'solo' => false,
        ];
    }

    public function solo(): self
    {
        return $this->state(fn (): array => [
            'solo' => true,
        ]);
    }
}
