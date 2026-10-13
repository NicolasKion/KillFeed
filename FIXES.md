# Fixes

Every Laravel bug the rewrite fixes instead of copying, with the test that
pins the new behavior, then the deliberate changes and what is left for
the owner to decide. Together with the test normalizations N1 to N6
(`tests/integration/common/golden.rs`) this is the complete list of
differences from the Laravel app on `main`.

## Bugs fixed

### A zKillboard hiccup broke the overlay

Laravel: the overlay page asks zKillboard for the character's latest
killmails on every view. `Http::retry(5)` throws after the fifth failed
try (the `failed()` check below it never runs), so whenever zKillboard
was slow or down every overlay answered 500, and Inertia's 10 second poll
put the error over the stream.

Now: the five tries stay, then the failure is logged and the overlay
renders from the stored killmails as usual.

Tests: `golden_pages::character_zkillboard_down`,
`zkillboard::a_failing_zkillboard_still_renders_the_overlay`.

### A character id beyond 64 bits answered 500

Laravel: `/{character}` matches any digit string, and an id that does
not fit PHP's int failed the controller's `int` parameter with a
TypeError, a 500.

Now: an id that is no EVE id answers 404, like an unknown one.

Tests: `golden_pages::character_overflow`,
`character_preview::an_id_beyond_64_bits_is_not_found`.

### An ESI connection error answered 500

Laravel: `CharacterService::resolve` turned a failed ESI answer into a
404 but did not catch a connection error (timeout, DNS), which answered
500 on the overlay and the preview.

Now: any ESI failure is a 404.

Test: `character_preview::an_unreachable_esi_is_not_found`.

### Downtime ran the killmail job anyway

Laravel: `NotDuringDowntime` released the job until 11:15 UTC when it
ran between 11:00 and 11:15, but then still called `$next`, so the job
ran during downtime (and failed against ESI) and was queued again for
11:15.

Now: a killmail job taken between 11:00 and 11:15 UTC waits until 11:15
without running.

Tests: `jobs::a_killmail_job_waits_out_the_downtime`,
`jobs::a_killmail_job_runs_right_after_the_downtime`.

### An empty zKillboard listing was never cached

Laravel: `if (! $killmails)` treated an empty listing (a character without
kills) as a cache miss, so every 10 second poll of such an overlay asked
zKillboard again, against their rate limit.

Now: an empty listing is kept for five minutes like any other.

Test: `zkillboard::an_empty_listing_is_cached_too`.

### The demo ships kept Tech I for a month after an ESI outage

Laravel: `ShipTypeService::demoShips` cached the whole list for 30 days,
including the Tech I fallbacks of lookups that failed, so one ESI hiccup
on the first setup page view hid every badge of the simulated feed for a
month (the per-type cache rightly skipped failures).

Now: only successful lookups are kept; a failed one is asked again on
the next view.

Test: `ship_type_service::a_failed_demo_ship_lookup_is_retried`.

### A character on both sides of a killmail failed its storage

Laravel: `updateOrCreate` matched involvements on character and role, so
a character who was both an attacker and the victim (or a stored
involvement whose role differed) hit the `(killmail_id, character_id)`
unique index; the transaction rolled back and the job failed, losing the
killmail.

Now: the first role stored wins (attackers first, as Laravel ordered
them), the killmail and the other involvements are stored.

Test: `process_killmail::a_character_on_both_sides_is_stored_once`.

### A broken firehose item stopped the listener for good

Laravel: a payload without the expected keys threw a TypeError in
`ListenForKills`; the command died before storing the sequence,
supervisor restarted it at the same sequence and it died again, forever.
A connection error ended the command the same way (it recovered only
because the next try usually worked).

Now: a broken item is logged and skipped, a connection error is retried
after the usual 6 seconds.

Tests: `listener::a_broken_item_is_skipped`,
`listener::a_connection_error_is_retried`.

### A broken zKillboard listing entry broke the overlay

Laravel: `zKillKillmail::fromArray` read every `zkb` key without a
fallback, so one listing entry missing a key (or an error object instead
of a list) answered 500.

Now: entries that do not parse are skipped.

Test: `zkillboard::a_broken_listing_entry_is_skipped`.

## Deliberate changes

### Horizon and Sanctum are gone

Horizon was installed but never ran (the queue is the database, there is
no Redis) and answered 403; Sanctum's `/sanctum/csrf-cookie` was never
called. Both answer 404 now.

Tests: `harness::the_horizon_and_sanctum_routes_are_gone`,
`golden_http::the_dropped_package_routes_are_not_found`.

### No session or XSRF cookies

There are no users and no forms; the `killfeed_session` and `XSRF-TOKEN`
cookies Laravel set on every page are not set any more.

Test: `golden_http::pages_set_no_cookies`.

### Overlays left open on the Laravel site reload

OBS overlays poll with Inertia's XHR (`X-Inertia: true`). Every such
request answers 409 with `X-Inertia-Location` (the full URL with its
query), which makes Inertia load the page in full, now from SvelteKit.
Laravel answered the same 409 for a stale asset version but ran the page
first; the 409 now comes before any side effect, and the reload that
follows does them.

Tests: `golden_http::inertia_stale`, `routes.spec.ts` "an Inertia poll
reloads into the new page".

### ESI's unversioned routes

ESI is called at `/characters/{id}`, `/universe/types/{id}` and
`/killmails/{id}/{hash}` with `X-Compatibility-Date: 2026-08-18` instead
of the deprecated `/latest/.../` paths; the fields used (`name`,
`dogma_attributes`, the killmail) have the same shape. On top, the
client keeps to `ESI_MAX_RPS` and stops calling ESI while it reports
fewer than 10 errors left in its error window, until the window resets.

Tests: `esi::requests_carry_the_compatibility_date`,
`esi::calls_stop_when_the_error_limit_runs_low`.

### The listener runs in the api, the stop command is gone

`app:listen-for-kills` is a task of the api (`LISTENER_ENABLED`); it
resumes from the stored sequence and stops after its current item on
SIGTERM, which replaces `app:stop-listening`.

Tests: `listener::resumes_after_the_stored_sequence`,
`listener::stops_when_asked`.

### `app:get-killmail` is gone

It dispatched `ProcessKillmail` without zKillboard data, which returns
right away: the command never did anything.

### A killmail's involvements in id order

Laravel loaded them without an order, and MySQL answered in whichever
index order its plan picked (by character id on some overlays, by id on
others). They come by id now. The overlay only counts them.

Tests: `golden_pages` (normalization N6), `golden_jobs`.

### One failed ESI lookup per setup page view

The setup page's thirteen demo ships each asked ESI three times half a
second apart while it failed (13 seconds per view in an outage, hidden
in Laravel by the month-long cache above). Once one lookup fails, the
rest of the view uses Tech I without asking. With the EVE services
switched off or ESI's error limit nearly used up, no lookup retries.

Test: `ship_type_service::a_failed_demo_ship_lookup_is_retried`.

### The EVE services can be switched off

`EVE_APIS_ENABLED=false` (the dev compose's default) fails every call to
ESI, zKillboard and R2Z2 at once, so a local stack never reaches them.

Tests: `character_preview::with_the_eve_services_off_nothing_is_asked`,
`demo_seed::the_sample_overlay_shows_a_week_of_killmails`.

### Housekeeping

Laravel's database cache and `failed_jobs` were never cleaned (the cache
held 25 MB of old zKillboard listings). Expired zKillboard listings and
failed jobs older than seven days are deleted every hour.

Test: `scheduler::housekeeping_drops_expired_listings_and_old_failed_jobs`.

## Needs owner decision

- The `characters` table keeps every character ever seen on a killmail
  (71k rows, never purged). Kept as it is.
- The listener's skip ahead: after ten 404s in a row it jumps to
  `sequence.json`, which at the head of the firehose is usually a few
  sequences back, so it reads killmails again. Kept (the jobs and the
  storage make the repeat a no-op); it could jump only forward.
- The overlay props carry each killmail's full `data` (every item) to
  every OBS source every 10 seconds; kept for parity, the overlay reads
  only a few fields.
- The old nginx config redirected any `*.killfeed.killmail.space` to the
  apex over http. The proxy serves only the apex (and redirects www); a
  wildcard would need a DNS challenge certificate in Traefik.
