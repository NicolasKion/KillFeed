<?php

namespace App\DTO;

readonly class zKillSimpleKillmail
{
    /**
     * Create a new class instance.
     */
    public function __construct(
        public int    $KillID,
        public string $action,
        public string $hash,
        public int    $character_id,
        public int    $ship_type_id,
        public int    $corporation_id,
        public string $url,
        public ?int   $alliance_id = null,
    )
    {
        //
    }

    public static function fromMessage(string $message): self
    {
        $data = json_decode($message, true);

        return self::fromArray($data);
    }

    public static function fromArray(array $data): self
    {
        return new self(
            KillID: $data['killID'],
            action: $data['action'],
            hash: $data['hash'],
            character_id: $data['victim']['character_id'],
            ship_type_id: $data['victim']['ship_type_id'],
            corporation_id: $data['victim']['corporation_id'],
            url: "https://zkillboard.com/kill/{$data['killID']}/",
            alliance_id: $data['victim']['alliance_id'] ?? null,
        );
    }
}
