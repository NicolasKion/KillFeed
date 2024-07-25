<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;

class Character extends Model
{
    public $incrementing = false;
    protected $fillable = [
        'id',
        'name',
        'listening_since',
    ];

    public function killmailInvolvements(): HasMany
    {
        return $this->hasMany(KillmailInvolvement::class, 'character_id');
    }

    public function listenNow(): void
    {
        $this->listening_since = now();
        $this->save();
    }

    protected function casts(): array
    {
        return [
            'listening_since' => 'datetime',
        ];
    }
}
