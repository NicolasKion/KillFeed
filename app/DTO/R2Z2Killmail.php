<?php

declare(strict_types=1);

namespace App\DTO;

final readonly class R2Z2Killmail
{
    private function __construct(
        private int $sequenceId,
        private int $killmailId,
        private string $killmailHash,
        private array $data,
    ) {}

    public static function fromArray(int $sequenceId, array $data): self
    {
        return new self(
            sequenceId: $sequenceId,
            killmailId: $data['killmail_id'],
            killmailHash: $data['hash'],
            data: $data,
        );
    }

    public function killmailId(): int
    {
        return $this->killmailId;
    }

    public function hash(): string
    {
        return $this->killmailHash;
    }

    public function sequenceId(): int
    {
        return $this->sequenceId;
    }

    public function toEsiKillmail(): EsiKillmail
    {
        return EsiKillmail::fromArray($this->data['esi']);
    }

    public function toZKillKillmail(): zKillKillmail
    {
        return zKillKillmail::fromArray([
            'killmail_id' => $this->data['killmail_id'],
            'zkb' => $this->data['zkb'],
        ]);
    }
}
