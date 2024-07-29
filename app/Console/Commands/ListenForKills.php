<?php

namespace App\Console\Commands;

use App\Jobs\ProcessKillmail;
use App\Models\Character;
use App\Services\CacheService;
use Exception;
use Illuminate\Console\Command;
use Illuminate\Support\Collection;
use Psr\Http\Message\RequestInterface;
use Psr\Http\Message\ResponseInterface;
use WebSocket;
use WebSocket\Client;
use WebSocket\Connection;

class ListenForKills extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:listen-for-kills';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Command description';

    protected Collection $connected_to_ids;

    function __construct()
    {
        parent::__construct();

        $this->connected_to_ids = collect();
    }

    /**
     * Execute the console command.
     */
    public function handle(CacheService $cacheService)
    {
        $started_at = now();
        $client = new WebSocket\Client("wss://zkillboard.com/websocket/");
        $client
            ->addMiddleware(new WebSocket\Middleware\CloseHandler())
            ->addMiddleware(new WebSocket\Middleware\PingResponder())
            ->addMiddleware(new WebSocket\Middleware\PingInterval(interval: 30));

        $client->onText(function (WebSocket\Client $client, WebSocket\Connection $connection, WebSocket\Message\Message $message) use ($cacheService) {
            $this->processMessage($message->getContent());
        })
            ->onHandshake(function (Client $client, Connection $connection, RequestInterface $request, ResponseInterface $response) {
                $this->connectToNewIds($client);
            })
            ->onTick(function (WebSocket\Client $client) use ($cacheService, &$started_at) {
                if ($started_at->diffInHours() >= 1) {
                    $this->disconnectFromAllIds($client);
                    $started_at = now();
                }
                $this->connectToNewIds($client);
                $this->disconnectFromInvalidIds($client);
            })
            ->onError(function (Client $client, Connection|null $connection, Exception $exception) {
                // Act on exception
                if (!$client->isRunning()) {
                    // Re-start if not running - will reconnect if necessary
                    $client->start();
                }
            });

        $client->start();
    }

    public function processMessage(string $message): void
    {
        $data = json_decode($message, true);

        $kill_id = $data['killID'] ?? $data['killmail_id'] ?? null;
        $hash = $data['hash'] ?? $data['zkb']['hash'] ?? null;

        if (!$kill_id || !$hash) {
            return;
        }

        $this->info("Processing killmail $kill_id");

        ProcessKillmail::dispatch($kill_id, $hash);
    }

    public function connectToNewIds(WebSocket\Client $client): Collection
    {
        $new_ids = $this->getCharactersToSubscribeTo()->diff($this->connected_to_ids);

        $new_ids->each(function (int $id) use ($client) {
            $this->info("Subscribing to $id");
            $client->text(sprintf('{"action":"sub","channel":"character:%d"}', $id));
            $this->connected_to_ids[] = $id;
        });

        return $new_ids;
    }

    public function getCharactersToSubscribeTo(): Collection
    {
        return Character::query()->where('listening_since', '>=', now()->subMinutes(10))->pluck('id');
    }

    function disconnectFromAllIds(WebSocket\Client $client): void
    {
        $this->connected_to_ids->each(function (int $id) use ($client) {
            $this->info("Unsubscribing from $id");
            $client->text(sprintf('{"action":"unsub","channel":"character:%d"}', $id));
        });

        $this->connected_to_ids = collect();
    }

    public function disconnectFromInvalidIds(WebSocket\Client $client): Collection
    {
        $invalid_ids = $this->connected_to_ids->diff($this->getCharactersToSubscribeTo());

        $invalid_ids->each(function (int $id) use ($client) {
            $this->info("Unsubscribing from $id");
            $client->text(sprintf('{"action":"unsub","channel":"character:%d"}', $id));
            $this->connected_to_ids = $this->connected_to_ids->diff([$id]);
        });

        return $invalid_ids;
    }

    public function listenToAllKillmails(WebSocket\Client $client): void
    {
        $client->text('{"action":"sub","channel":"killstream"}');
    }

    public function unsubscribeFromAllKillmails(WebSocket\Client $client): void
    {
        $client->text('{"action":"unsub","channel":"killstream"}');
    }
}
