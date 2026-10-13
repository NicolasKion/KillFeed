# KillFeed

[![CI](https://github.com/TimKunze96/KillFeed/actions/workflows/ci.yml/badge.svg)](https://github.com/TimKunze96/KillFeed/actions/workflows/ci.yml)
[![Website](https://img.shields.io/website?url=https%3A%2F%2Fkillfeed.killmail.space&label=killfeed.killmail.space)](https://killfeed.killmail.space)
[![Rust](https://img.shields.io/badge/Rust-Axum-DEA584?logo=rust&logoColor=white)](https://github.com/tokio-rs/axum)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-Svelte%205-FF3E00?logo=svelte&logoColor=white)](https://svelte.dev/docs/kit)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-18-4169E1?logo=postgresql&logoColor=white)](https://www.postgresql.org)

A live killfeed overlay for EVE Online streamers: paste one URL into an
OBS browser source and your kills and losses show up on stream.

## Features

- A setup page that builds the overlay URL for a character, with a live
  preview and a simulated feed to try the options on
- The overlay: killmail cards with ship renders, ISK values, solo or the
  number of pilots, meta group badges (Tech II, Tech III, faction,
  storyline, officer), a `+N` tile for what does not fit, and an optional
  summary of ISK destroyed, lost and efficiency
- Options in the URL: cards shown, time window, summary panel, badges
- New killmails within seconds from zKillboard's R2Z2 firehose, for every
  character with an overlay open, plus a backfill of each character's
  latest killmails from zKillboard on every overlay refresh
- A full-page demo with simulated killmails
- Killmails older than a week are purged every night

## Stack

- API: Rust, [Axum](https://github.com/tokio-rs/axum) and
  [sqlx](https://github.com/launchbadge/sqlx) on Postgres. The firehose
  listener, the job workers and the nightly purge run inside the API.
- Frontend: [SvelteKit](https://svelte.dev/docs/kit) (SSR, adapter-node),
  Svelte 5 and Tailwind CSS 4
- Data from [ESI](https://esi.evetech.net), [zKillboard](https://zkillboard.com)
  and its R2Z2 firehose; images from images.evetech.net

One origin serves both: Caddy (or the Vite dev proxy) sends `/api/*`,
`/characters/*` and `/up` to Axum, and page loads to SvelteKit.
`frontend/proxy-paths.ts` and `deploy/Caddyfile` hold that list.

## Getting started

The whole stack (Postgres, API, SvelteKit, Caddy) on
http://localhost:5316:

```bash
docker compose up --build -d
```

The local stack never calls ESI, zKillboard or R2Z2. The `demo-seed`
service (safe to repeat) stores a sample character with a week of
killmails, so http://localhost:5316/2112625428?summary=1 shows an
overlay. To work against the real services:

```bash
EVE_APIS_ENABLED=true docker compose up -d api              # ESI and zKillboard
EVE_APIS_ENABLED=true LISTENER_ENABLED=true docker compose up -d api   # and the firehose
```

Only one firehose listener should run at a time anywhere, so leave
`LISTENER_ENABLED` off unless you need it.

Or natively, with only Postgres in docker:

```bash
docker compose up -d postgres     # Postgres on 127.0.0.1:5446
cargo run                         # API on 127.0.0.1:3000
cd frontend && npm install && npm run dev
```

Configuration lives in `.env` (see `.env.example`); everything has a
working local default.

## Testing

```bash
docker compose up -d postgres
cargo test
cd frontend && npm run check && npm run lint && npm test
docker compose up --build -d && cd frontend && npm run test:e2e
```

The Rust suites drive the real router on their own `killfeed_test`
database with a fixed clock; ESI, zKillboard and R2Z2 are a local mock
server. They pin every page's props, the firehose listener, the job
queue and the storage of killmails. The vitest suites cover the overlay's
logic and render it in a headless Chromium, and Playwright runs against
the composed stack on port 5316. `scripts/parity` prints the passing
tests over the total for all three.

## Deployment

Coolify deploys `compose.prod.yml` on the server. A merge to main runs
`.github/workflows/release.yml`, which builds the api, frontend and proxy
images, pushes them to `ghcr.io/nicolaskion/killfeed-*` (`latest` and the
commit) and calls Coolify's deploy webhook, so Coolify pulls and restarts
them. Only the proxy has a domain (https://killfeed.killmail.space);
Traefik terminates TLS in front of it. `DATABASE_URL` (the shared
Postgres on the `coolify` network) and `LISTENER_ENABLED=true` are
Coolify environment variables. The API runs its migrations on startup
(`migrate` applies them on its own and exits) and stops the listener
cleanly on shutdown, so it resumes where it left off.
`purge_old_killmails` runs the nightly purge by hand.

## Layout

```
src/                 the API: router, listener, jobs, scheduler, ESI and zKillboard clients
src/bin/             migrate, purge_old_killmails, seed_demo
migrations/          the Postgres schema
tests/integration/   the Rust suites, one binary
tests/fixtures/      the dataset and EVE answers the suites use
frontend/            the SvelteKit app and its Playwright suite (e2e/)
deploy/              the proxy: Caddyfile and image, local and production
```

## License

MIT
