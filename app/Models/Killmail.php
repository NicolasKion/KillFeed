<?php

declare(strict_types=1);

namespace App\Models;

use Database\Factories\KillmailFactory;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Support\Carbon;

/**
 * @property int $id
 * @property int $character_id
 * @property string $hash
 * @property array $data
 * @property Carbon $time
 * @property int $solarsystem_id
 * @property int|null $corporation_id
 * @property int|null $alliance_id
 * @property int $ship_type_id
 * @property int|null $ship_meta_group_id
 * @property int $damage_taken
 * @property float $value
 * @property bool $solo
 * @property-read Character $character
 * @property-read Collection<int, KillmailInvolvement> $killmailInvolvements
 */
final class Killmail extends Model
{
    /** @use HasFactory<KillmailFactory> */
    use HasFactory;

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
        'ship_meta_group_id',
        'damage_taken',
        'value',
        'solo',
    ];

    /**
     * @return BelongsTo<Character, $this>
     */
    public function character(): BelongsTo
    {
        return $this->belongsTo(Character::class);
    }

    /**
     * @return HasMany<KillmailInvolvement, $this>
     */
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
