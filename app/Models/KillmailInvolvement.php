<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class KillmailInvolvement extends Model
{
    protected $fillable = [
        'killmail_id',
        'character_id',
        'is_attacker',
    ];

    public function character(): BelongsTo
    {
        return $this->belongsTo(Character::class);
    }

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
