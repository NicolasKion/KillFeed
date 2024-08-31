<?php

namespace App\DTO;

readonly class RedisQKillmail
{
    function __construct(
        public zKillKillmail $zkb,
        public EsiKillmail   $esi,
    )
    {

    }

    public static function fromArray(array $data): self
    {
        return new self(
            zkb: zKillKillmail::fromArray($data['package']),
            esi: EsiKillmail::fromArray($data['package']['killmail']),
        );
    }
}
