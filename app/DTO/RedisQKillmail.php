<?php

declare(strict_types=1);

namespace App\DTO;

final readonly class RedisQKillmail
{
    public function __construct(
        public zKillKillmail $zkb,
        public int $killmail_id,
    ) {}

    public static function fromArray(array $data): self
    {
        return new self(
            zkb: zKillKillmail::fromArray($data['package']),
            killmail_id: $data['package']['killID'],
        );
    }
}
