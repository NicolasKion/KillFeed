<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Models\Character;
use App\Models\Killmail;
use App\Models\KillmailInvolvement;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<KillmailInvolvement>
 */
final class KillmailInvolvementFactory extends Factory
{
    protected $model = KillmailInvolvement::class;

    public function definition(): array
    {
        return [
            'killmail_id' => Killmail::factory(),
            'character_id' => Character::factory(),
            'is_attacker' => false,
        ];
    }

    public function attacker(): self
    {
        return $this->state(fn (): array => [
            'is_attacker' => true,
        ]);
    }
}
