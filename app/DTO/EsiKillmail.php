<?php

namespace App\DTO;


/**
 * @property-read EsiKillmailAttacker[] $attackers
 */
readonly class EsiKillmail
{
    public function __construct(
        public int               $killmail_id,
        public string            $killmail_time,
        public ?int              $moon_id,
        public int               $solar_system_id,
        public EsiKillmailVictim $victim,
        public array             $attackers = [], // AttackerDTO[]
        public ?int              $war_id = null
    )
    {
    }

    public static function fromArray(array $data): self
    {
        return new self(
            killmail_id: $data['killmail_id'],
            killmail_time: $data['killmail_time'],
            moon_id: $data['moon_id'] ?? null,
            solar_system_id: $data['solar_system_id'],
            victim: EsiKillmailVictim::fromArray($data['victim']),
            attackers: array_map(fn(array $attacker) => EsiKillmailAttacker::fromArray($attacker), $data['attackers']),
            war_id: $data['war_id'] ?? null
        );
    }
}
