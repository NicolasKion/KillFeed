# killfeed.killmail.space

KillFeed: an OBS browser-source overlay for EVE Online streamers. A setup
page builds an overlay URL (`/{character}?count=&hours=&summary=&meta=`)
whose page polls itself every 10 seconds; a listener on zKillboard's R2Z2
firehose queues the killmails of characters with an open overlay, and
every overlay view backfills the character's latest killmails from
zKillboard. A Rust API (Axum + Postgres via sqlx) serving JSON, with the
listener, the job workers and the scheduler as tasks of the api, and a
SvelteKit frontend (SSR, adapter-node, Svelte 5, Tailwind 4) in
`frontend/`. One shared origin (the Vite dev proxy locally, Caddy in
docker and production) routes backend paths to Axum and page GETs to
SvelteKit. Never point the browser at Axum's port directly. The setup
mirrors Medico (`../Medico`); MutaMarket (`../MutaMarket`) is the
reference for ESI work.

## Testing rules

- Every change lands with its tests in the same commit.
- Tests are precise: exact JSON key sets at every level
  (`common::assert_json_eq`), exact messages, statuses and orderings,
  with the expected values written out in the test.
- Behavior tests drive the real router (`oneshot`) on the dedicated
  `killfeed_test` database (`db::test_pool()`, created and migrated on
  first use, `TEST_DATABASE_URL` to override) with a fixed clock
  (`clock::Clock`). ESI, zKillboard and R2Z2 are one local mock on an
  ephemeral port (`tests/integration/common/mock_eve.rs`), never stubs of
  our own code.
- `tests/fixtures/dataset.json` is five characters and six killmails with
  the edge cases between them, `tests/fixtures/eve.json` what the mock
  answers for them (`TestApp::with_dataset()`, clock at `DATASET_NOW`).
  `pages.rs` pins every page's props on it with the rows and requests a
  view leaves, `routes.rs` the other answers.
- Integration suites are modules of the single `tests/integration` binary
  and run single-threaded (`.cargo/config.toml`): they truncate the shared
  tables. Run one suite with `cargo test <module>::`.
- Frontend logic has vitest suites next to the code (`*.test.ts`).
  Playwright (`frontend/e2e`) runs against the composed stack and seeds
  through `docker exec killfeed-postgres psql`.
- Before committing: `cargo fmt`, `cargo clippy --all-targets` (zero
  warnings), `cargo test`, and in `frontend/` `npm run check`,
  `npm run lint`, `npm run format:check`, `npm test`.

## Layout and commands

- `src/server/` the router and the layers; `src/listener.rs` the R2Z2
  listener; `src/jobs.rs` the queue; `src/scheduler.rs` the nightly
  purge; `migrations/` the schema; `deploy/` the proxy (Caddyfile and
  image, local and production); `frontend/proxy-paths.ts` the routing
  contract, mirrored by `deploy/Caddyfile`.
- Development: `docker compose up -d postgres`, `cargo run` (API on
  `BIND_ADDR`, default 127.0.0.1:3000, see `.env.example`) and
  `npm run dev` in `frontend/`.
- Whole stack: `docker compose up --build` on http://localhost:5316.
  Ports are fixed: Postgres 5446, stack 5316.
  The stack never calls ESI, zKillboard or R2Z2 unless
  `EVE_APIS_ENABLED=true` (and `LISTENER_ENABLED=true` for the
  firehose); `demo-seed` (the api image's `seed_demo`, safe to repeat)
  stores sample characters and killmails and the demo ships' meta groups.
  Playwright truncates the same database, so run
  `docker compose run --rm demo-seed` again after an e2e run.
- Local preview on https://killfeed.test (portal):
  `ORIGIN=https://killfeed.test docker compose up --build -d`.
- Frontend: pages load their props from `/api/pages/<path>`
  (`#lib/server/api.ts`); the overlay is `#lib/components/killfeed-overlay.svelte`.
- Deploy: a merge to main builds the api, frontend and proxy images in
  GitHub Actions (`.github/workflows/release.yml`), pushes them to
  `ghcr.io/nicolaskion/killfeed-*` and has Coolify redeploy
  `compose.prod.yml`.
