# Ported tests

Every test of the Laravel app (`main`: `tests/Unit`, `tests/Feature`,
`tests/Browser`, Pest) and the test that replaces it. Feature and unit
tests run against the real router and database in `tests/integration`
(one module per PHP file, same names); browser tests are Playwright specs
in `frontend/e2e` against the composed stack. The PHP tests faked ESI and
zKillboard with `Http::fake`; the ports use the mock server of
`tests/integration/common/mock_eve.rs`, and the browser ports seed the
character a fake would have named. Where a PHP test pinned a Laravel bug,
the port pins the fix and points at FIXES.md.

| PHP test | New test |
| --- | --- |
| Unit/ExampleTest "that true is true" | dropped: asserts nothing |
| Unit/R2Z2KillmailTest "it parses an R2Z2 payload into killmail id, hash, and sequence" | `r2z2_killmail::it_parses_an_r2z2_payload_into_killmail_id_hash_and_sequence` |
| Unit/R2Z2KillmailTest "it converts to EsiKillmail" | `r2z2_killmail::it_converts_to_esi_killmail` |
| Unit/R2Z2KillmailTest "it converts to zKillKillmail" | `r2z2_killmail::it_converts_to_zkill_killmail` |
| Feature/CharacterPreviewTest "character preview returns resolved character data" | `character_preview::character_preview_returns_resolved_character_data` |
| Feature/CharacterPreviewTest "character preview route only accepts numeric ids" | `character_preview::character_preview_route_only_accepts_numeric_ids` |
| Feature/DemoTest "demo page renders with defaults" | `demo::demo_page_renders_with_defaults` |
| Feature/DemoTest "demo page clamps parameters to configured maximums" | `demo::demo_page_clamps_parameters_to_configured_maximums` |
| Feature/ExampleTest "it returns a successful response" | `example::it_returns_a_successful_response` |
| Feature/KillmailFeedTest "killmails stored before meta groups were tracked heal when served" | `killmail_feed::killmails_stored_before_meta_groups_were_tracked_heal_when_served` |
| Feature/ProcessKillmailTest "it stores a killmail when full data is provided without fetching it" | `process_killmail::it_stores_a_killmail_when_full_data_is_provided_without_fetching_it` |
| Feature/ProcessKillmailTest "it fetches ESI killmail when esiKillmail is null" | `process_killmail::it_fetches_esi_killmail_when_esi_killmail_is_null` |
| Feature/ProcessKillmailTest "it returns early when zKillKillmail is null" | `process_killmail::it_returns_early_when_zkill_killmail_is_null` |
| Feature/ShipTypeServiceTest "it resolves faction ships from the meta group attribute" | `ship_type_service::it_resolves_faction_ships_from_the_meta_group_attribute` |
| Feature/ShipTypeServiceTest "it resolves Tech II and Tech III hulls from the tech level attribute" | `ship_type_service::it_resolves_tech_ii_and_tech_iii_hulls_from_the_tech_level_attribute` |
| Feature/ShipTypeServiceTest "it defaults to Tech I when no relevant attributes are present" | `ship_type_service::it_defaults_to_tech_i_when_no_relevant_attributes_are_present` |
| Feature/ShipTypeServiceTest "it caches the meta group per type id" | `ship_type_service::it_caches_the_meta_group_per_type_id` |
| Feature/ShipTypeServiceTest "it falls back to Tech I without caching when ESI fails" | `ship_type_service::it_falls_back_to_tech_i_without_caching_when_esi_fails` |
| Feature/UserAgentTest "outgoing requests identify the app and its maintainer" | `user_agent::outgoing_requests_identify_the_app_and_its_maintainer` |
| Browser/CharacterPageTest "it renders the killfeed overlay for a tracked character" | `character-page.spec.ts` "renders the killfeed overlay for a tracked character" |
| Browser/CharacterPageTest "it shows the summary panel with ISK totals when requested" | `character-page.spec.ts` "shows the summary panel with ISK totals when requested" |
| Browser/CharacterPageTest "it marks the character as being listened to" | `character-page.spec.ts` "marks the character as being listened to", `character_page::the_overlay_marks_the_character_as_listening` |
| Browser/DemoPageTest "it renders the simulated overlay" | `demo-page.spec.ts` "renders the simulated overlay" |
| Browser/DemoPageTest "it shows the summary panel when requested" | `demo-page.spec.ts` "shows the summary panel when requested" |
| Browser/HomePageTest "it renders the overlay setup page" | `home-page.spec.ts` "renders the overlay setup page" |
| Browser/HomePageTest "it runs the simulated feed until a character is entered" | `home-page.spec.ts` "runs the simulated feed until a character is entered" |
| Browser/HomePageTest "it previews a character after entering an id" | `home-page.spec.ts` "previews a character after entering an id" |

## New test areas

- Golden comparisons with the Laravel reference: every page's props,
  rows and outgoing requests (`golden_pages`, including the PHP query
  parsing table of `demo-query-*`), the non-page answers (`golden_http`),
  the queue storing killmails like Laravel (`golden_jobs`), the page
  heads (`seo.spec.ts`) and screenshots (`visual.spec.ts`).
- The firehose listener against a mock R2Z2 (`listener`): start and
  resume, the 404 wait and skip ahead, retries, the watched characters,
  broken items, shutdown.
- The queue (`jobs`): one pending job per killmail, failed jobs kept,
  the downtime, jobs of a stopped api.
- The zKillboard backfill (`zkillboard`), the ESI client (`esi`), the
  schedule and purge (`scheduler`), the overlay's side effects and totals
  (`character_page`), the preview's failures (`character_preview`), the
  storage's repeats and odd killmails (`process_killmail`), PHP's
  `intval` and boolean filter (`options`), the route contract
  (`harness`, `routes.spec.ts`).
- Frontend logic in vitest: the overlay's arithmetic, the setup page's
  URL building, the simulated feed with a seeded generator, the tween,
  the image URLs and badges, the overlay component in a browser, the
  routing contract and the Inertia reload.
