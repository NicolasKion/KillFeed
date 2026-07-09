# KillFeed

[![CI](https://github.com/TimKunze96/KillFeed/actions/workflows/ci.yml/badge.svg)](https://github.com/TimKunze96/KillFeed/actions/workflows/ci.yml)

A real-time **EVE Online killmail overlay for streamers**. KillFeed tracks a character's kills and losses live and renders them as a compact, OBS-ready browser-source overlay — killmail cards with ship renders and ISK values, plus an optional summary panel with ISK destroyed, ISK lost, and efficiency.

Killmails are streamed from the [zKillboard R2Z2 firehose](https://github.com/zKillboard/RedisQ) and enriched with data from [ESI](https://esi.evetech.net/), EVE's official API.

## How it works

1. A long-running listener (`app:listen-for-kills`) consumes the R2Z2 firehose and dispatches a job for every killmail involving a tracked character.
2. Jobs fetch the full killmail from ESI, compute its value, and store it with all character involvements.
3. The overlay page (`/{characterId}`) polls every 10 seconds and renders the latest killmails inside the configured time window.

Setup happens on the home page: enter a character ID, tweak the options, and copy the generated overlay URL into an OBS browser source.

## Stack

- [Laravel 13](https://laravel.com) on PHP 8.4, with [Horizon](https://laravel.com/docs/horizon) + Redis for queues
- [Inertia v3](https://inertiajs.com) with [Vue 3](https://vuejs.org), TypeScript, and SSR
- [Tailwind CSS 4](https://tailwindcss.com) and [Vite 8](https://vite.dev)
- [Wayfinder](https://github.com/laravel/wayfinder) for type-safe routes in the frontend

## Development

Requirements: PHP 8.4 (with `pcntl`), Composer, Node 22+, MySQL, Redis.

```bash
composer install
npm install

cp .env.example .env
php artisan key:generate
php artisan migrate

# In separate terminals:
php artisan serve          # app server
npm run dev                # Vite dev server
php artisan horizon        # queue worker
php artisan app:listen-for-kills   # killmail firehose listener
```

### Console commands

| Command                        | Purpose                                                      |
| ------------------------------ | ------------------------------------------------------------ |
| `app:listen-for-kills`         | Listen to the R2Z2 firehose for new killmails                |
| `app:stop-listening`           | Signal the listener to shut down gracefully                  |
| `app:get-killmail {id} {hash}` | Fetch and process a single killmail manually                 |
| `app:purge-old-killmails`      | Purge killmails older than the retention window (runs daily) |

### Configuration

Killfeed-specific options live in `config/killfeed.php`:

| Env variable                  | Default    | Purpose                                     |
| ----------------------------- | ---------- | ------------------------------------------- |
| `KILLMAIL_MAX_VISIBLE`        | `50`       | Maximum killmail cards shown in the overlay |
| `KILLMAIL_MAX_AGE_HOURS`      | `168`      | Maximum age of displayed killmails (hours)  |
| `KILLFEED_DISCORD_INVITE_URL` | –          | Discord invite shown on the setup page      |
| `R2Z2_BASE_URL`               | zKillboard | Base URL of the R2Z2 firehose               |

## Testing

The test suite uses [Pest 4](https://pestphp.com), including real-browser tests via Playwright.

```bash
# One-time setup for browser tests
npx playwright install chromium

# Browser tests run against the built frontend
npm run build

php artisan test                      # everything
php artisan test --testsuite=Unit    # unit tests only
php artisan test --testsuite=Feature # feature tests only
php artisan test --testsuite=Browser # browser tests only
```

## Code quality

| Check           | Command                                        |
| --------------- | ---------------------------------------------- |
| PHP code style  | `composer lint` (fix: `composer lint:fix`)     |
| Static analysis | `composer analyse`                             |
| Refactoring     | `composer refactor`                            |
| JS/Vue lint     | `npm run lint` (fix: `npm run lint:fix`)       |
| Formatting      | `npm run format:check` (fix: `npm run format`) |
| Type check      | `npm run typecheck`                            |

All of these run in CI on every push and pull request (see `.github/workflows/ci.yml`).

## License

MIT
