<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\Carbon;
use Database\Factories\CharacterFactory;
use Illuminate\Database\Eloquent\Attributes\Scope;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * @property int $id
 * @property string $name
 * @property Carbon|string $listening_since
 * @property-read Collection<int, KillmailInvolvement> $killmailInvolvements
 */
final class Character extends Model
{
    /** @use HasFactory<CharacterFactory> */
    use HasFactory;

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

    /**
     * Characters with an overlay open recently enough to monitor.
     *
     * @param  Builder<Character>  $query
     */
    #[Scope]
    protected function listening(Builder $query): void
    {
        $query->where('listening_since', '>', now()->subMinutes(5));
    }

    protected function casts(): array
    {
        return [
            'listening_since' => 'datetime',
        ];
    }
}
