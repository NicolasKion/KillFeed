<?php

declare(strict_types=1);

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * @property int $killmail_id
 * @property int $character_id
 * @property bool $is_attacker
 * @property-read Character $character
 * @property-read Killmail $killmail
 */
final class KillmailInvolvement extends Model
{
    protected $fillable = [
        'killmail_id',
        'character_id',
        'is_attacker',
    ];

    /**
     * @return BelongsTo<Character, $this>
     */
    public function character(): BelongsTo
    {
        return $this->belongsTo(Character::class);
    }

    /**
     * @return BelongsTo<Killmail, $this>
     */
    public function killmail(): BelongsTo
    {
        return $this->belongsTo(Killmail::class);
    }

    protected function casts(): array
    {
        return [
            'is_attacker' => 'boolean',
        ];
    }
}
