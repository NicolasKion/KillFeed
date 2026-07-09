<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Models\Character;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<Character>
 */
final class CharacterFactory extends Factory
{
    protected $model = Character::class;

    public function definition(): array
    {
        return [
            'id' => fake()->unique()->numberBetween(90_000_000, 100_000_000),
            'name' => fake()->name(),
            'listening_since' => null,
        ];
    }

    public function listening(): self
    {
        return $this->state(fn (): array => [
            'listening_since' => now(),
        ]);
    }
}
