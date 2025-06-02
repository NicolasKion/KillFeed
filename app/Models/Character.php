<?php

namespace App\Models;

use Carbon\Carbon;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * @property int $id
 * @property string $name
 * @property Carbon|string $listening_since
 * @property-read Collection<int, KillmailInvolvement> $killmailInvolvements
 */
class Character extends Model
{
    public $incrementing = false;
    protected $fillable = [
        'id',
        'name',
        'listening_since',
    ];

    /**
     * @return HasMany<KillmailInvolvement, $this>
     */
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
