<?php

namespace App\DTO;

readonly class RedisQKillmail
{
    function __construct(
        public zKillKillmail $zkb,
        public int $killmail_id,
    )
    {

    }

    public static function fromArray(array $data): self
    {
        return new self(
            zkb: zKillKillmail::fromArray($data['package']),
            killmail_id: $data['package']['killID'],
        );
    }
}
