<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

class Killmail extends Model
{
    public $incrementing = false;
    protected $fillable = [
        'id',
        'character_id',
        'hash',
        'data',
        'time',
        'solarsystem_id',
        'corporation_id',
        'alliance_id',
        'ship_type_id',
        'damage_taken',
        'value',
        'solo',
    ];

    public function character(): BelongsTo
    {
        return $this->belongsTo(Character::class);
    }

    public function killmailInvolvements(): HasMany
    {
        return $this->hasMany(KillmailInvolvement::class, 'killmail_id');
    }

    protected function casts(): array
    {
        return [
            'data' => 'collection',
            'time' => 'datetime',
        ];
    }
}
