//! The props of every page on the dataset (`/api/pages/<path>`), the
//! rows a request leaves behind (characters, queued jobs) and what it
//! asks ESI and zKillboard, with the clock at `DATASET_NOW` and the mock
//! answering tests/fixtures/eve.json.

use serde_json::{Value, json};

use crate::common::{TestApp, assert_json_eq};

/// The props endpoint of a page path.
fn api_path(path: &str) -> String {
    match path.split_once('?') {
        Some(("/", query)) => format!("/api/pages/home?{query}"),
        None if path == "/" => "/api/pages/home".to_owned(),
        _ => format!("/api/pages{path}"),
    }
}

/// The queued jobs: kind, killmail, and whether the payload carries the
/// zKillboard and ESI answers.
async fn queued(app: &TestApp) -> Value {
    let rows: Vec<(String, i64, bool, bool)> = sqlx::query_as(
        "select kind, killmail_id, coalesce(jsonb_typeof(payload->'zkb') = 'object', false),
                coalesce(jsonb_typeof(payload->'esi') = 'object', false)
         from jobs order by id",
    )
    .fetch_all(&app.pool)
    .await
    .expect("jobs read");
    Value::Array(
        rows.into_iter()
            .map(|(kind, killmail_id, with_zkill, with_esi)| {
                json!({ "job": kind, "killmail_id": killmail_id,
                        "with_zkill": with_zkill, "with_esi": with_esi })
            })
            .collect(),
    )
}

/// A zKillboard backfill job for a killmail not stored yet.
fn backfill(killmail_id: i64) -> Value {
    json!({ "job": "process_killmail", "killmail_id": killmail_id, "with_zkill": true, "with_esi": false })
}

/// The meta group heal of a stored killmail without one.
fn meta_group(killmail_id: i64) -> Value {
    json!({ "job": "backfill_meta_group", "killmail_id": killmail_id, "with_zkill": false, "with_esi": false })
}

/// The moment the requests run (`DATASET_NOW`), as the rows write it.
const NOW: &str = "2026-10-13T12:15:33.000000Z";

/// The dataset's characters after a request, by id: the ones in
/// `listening` were marked as listening now, `named` got their name from
/// ESI.
fn characters(listening: &[i64], named: &[(i64, &str)]) -> Value {
    let mut rows = json!([
        {
            "created_at": "2026-10-11T10:15:33.000000Z",
            "id": 90000001,
            "listening_since": null,
            "name": "Quiet Pilot",
            "updated_at": "2026-10-11T10:15:33.000000Z"
        },
        {
            "created_at": "2026-10-11T10:15:33.000000Z",
            "id": 93813310,
            "listening_since": null,
            "name": null,
            "updated_at": "2026-10-11T10:15:33.000000Z"
        },
        {
            "created_at": "2026-10-11T10:15:33.000000Z",
            "id": 95000002,
            "listening_since": "2026-10-13T12:05:33.000000Z",
            "name": "Old Listener",
            "updated_at": "2026-10-13T12:05:33.000000Z"
        },
        {
            "created_at": "2026-10-11T10:15:33.000000Z",
            "id": 95465499,
            "listening_since": null,
            "name": "Attacker One",
            "updated_at": "2026-10-11T10:15:33.000000Z"
        },
        {
            "created_at": "2026-10-11T10:15:33.000000Z",
            "id": 2112625428,
            "listening_since": null,
            "name": "Capsuleer Prime",
            "updated_at": "2026-10-11T10:15:33.000000Z"
        }
    ]);
    for row in rows.as_array_mut().expect("rows") {
        let id = row["id"].as_i64().expect("an id");
        if listening.contains(&id) {
            row["listening_since"] = json!(NOW);
            row["updated_at"] = json!(NOW);
        }
        if let Some((_, name)) = named.iter().find(|(named, _)| *named == id) {
            row["name"] = json!(name);
        }
    }
    rows
}

/// A dataset character as the overlay's props carry it.
fn character_row(characters: &Value, id: i64) -> Value {
    characters
        .as_array()
        .expect("rows")
        .iter()
        .find(|row| row["id"] == id)
        .cloned()
        .expect("a dataset character")
}

/// A dataset killmail as the overlay serves it, with its involvements.
fn killmail(id: i64) -> Value {
    match id {
        120000001 => json!({
            "alliance_id": null,
            "character_id": 93813310,
            "corporation_id": 98000001,
            "created_at": "2026-10-13T11:46:33.000000Z",
            "damage_taken": 2400,
            "data": {
                "attackers": [
                    {
                        "alliance_id": 99000001,
                        "character_id": 2112625428,
                        "corporation_id": 98000002,
                        "damage_done": 2400,
                        "faction_id": null,
                        "final_blow": true,
                        "security_status": -2.5,
                        "ship_type_id": 587,
                        "weapon_type_id": 3170
                    }
                ],
                "killmail_id": 120000001,
                "killmail_time": "2026-10-13T11:45:33Z",
                "moon_id": null,
                "solar_system_id": 30002187,
                "victim": {
                    "alliance_id": null,
                    "character_id": 93813310,
                    "corporation_id": 98000001,
                    "damage_taken": 2400,
                    "faction_id": null,
                    "items": [
                        {
                            "flag": 27,
                            "item_type_id": 2873,
                            "items": [],
                            "quantity_destroyed": 1,
                            "quantity_dropped": null,
                            "singleton": 0
                        },
                        {
                            "flag": 5,
                            "item_type_id": 21894,
                            "items": [],
                            "quantity_destroyed": null,
                            "quantity_dropped": 120,
                            "singleton": 0
                        },
                        {
                            "flag": 5,
                            "item_type_id": 3020,
                            "items": [
                                {
                                    "flag": 5,
                                    "item_type_id": 2205,
                                    "items": [],
                                    "quantity_destroyed": null,
                                    "quantity_dropped": 2,
                                    "singleton": 0
                                }
                            ],
                            "quantity_destroyed": null,
                            "quantity_dropped": null,
                            "singleton": 0
                        }
                    ],
                    "position": {
                        "x": -107303362560_i64,
                        "y": -18744975360.5,
                        "z": 436489052160.25
                    },
                    "ship_type_id": 11176
                },
                "war_id": null
            },
            "hash": "hash120000001",
            "id": 120000001,
            "killmail_involvements": [
                {
                    "character_id": 2112625428,
                    "created_at": "2026-10-13T11:46:33.000000Z",
                    "id": 1,
                    "is_attacker": true,
                    "killmail_id": 120000001,
                    "updated_at": "2026-10-13T11:46:33.000000Z"
                },
                {
                    "character_id": 93813310,
                    "created_at": "2026-10-13T11:46:33.000000Z",
                    "id": 2,
                    "is_attacker": false,
                    "killmail_id": 120000001,
                    "updated_at": "2026-10-13T11:46:33.000000Z"
                }
            ],
            "ship_meta_group_id": 2,
            "ship_type_id": 11176,
            "solarsystem_id": 30002187,
            "solo": 1,
            "time": "2026-10-13T11:45:33.000000Z",
            "updated_at": "2026-10-13T11:46:33.000000Z",
            "value": 1000000
        }),
        120000002 => json!({
            "alliance_id": 99000001,
            "character_id": 2112625428,
            "corporation_id": 98000002,
            "created_at": "2026-10-13T10:16:33.000000Z",
            "damage_taken": 98000,
            "data": {
                "attackers": [
                    {
                        "alliance_id": null,
                        "character_id": 95465499,
                        "corporation_id": 98000003,
                        "damage_done": 60000,
                        "faction_id": null,
                        "final_blow": true,
                        "security_status": 5,
                        "ship_type_id": 24698,
                        "weapon_type_id": 3170
                    },
                    {
                        "alliance_id": null,
                        "character_id": 93813310,
                        "corporation_id": 98000001,
                        "damage_done": 38000,
                        "faction_id": null,
                        "final_blow": false,
                        "security_status": 0.1,
                        "ship_type_id": 626,
                        "weapon_type_id": 3170
                    },
                    {
                        "alliance_id": null,
                        "character_id": null,
                        "corporation_id": null,
                        "damage_done": 0,
                        "faction_id": 500010,
                        "final_blow": false,
                        "security_status": 0,
                        "ship_type_id": 23913,
                        "weapon_type_id": null
                    }
                ],
                "killmail_id": 120000002,
                "killmail_time": "2026-10-13T10:15:33Z",
                "moon_id": null,
                "solar_system_id": 30000142,
                "victim": {
                    "alliance_id": 99000001,
                    "character_id": 2112625428,
                    "corporation_id": 98000002,
                    "damage_taken": 98000,
                    "faction_id": null,
                    "items": [
                        {
                            "flag": 11,
                            "item_type_id": 3170,
                            "items": [],
                            "quantity_destroyed": 1,
                            "quantity_dropped": null,
                            "singleton": 1
                        }
                    ],
                    "position": null,
                    "ship_type_id": 17738
                },
                "war_id": null
            },
            "hash": "hash120000002",
            "id": 120000002,
            "killmail_involvements": [
                {
                    "character_id": 95465499,
                    "created_at": "2026-10-13T10:16:33.000000Z",
                    "id": 3,
                    "is_attacker": true,
                    "killmail_id": 120000002,
                    "updated_at": "2026-10-13T10:16:33.000000Z"
                },
                {
                    "character_id": 93813310,
                    "created_at": "2026-10-13T10:16:33.000000Z",
                    "id": 4,
                    "is_attacker": true,
                    "killmail_id": 120000002,
                    "updated_at": "2026-10-13T10:16:33.000000Z"
                },
                {
                    "character_id": 2112625428,
                    "created_at": "2026-10-13T10:16:33.000000Z",
                    "id": 5,
                    "is_attacker": false,
                    "killmail_id": 120000002,
                    "updated_at": "2026-10-13T10:16:33.000000Z"
                }
            ],
            "ship_meta_group_id": 4,
            "ship_type_id": 17738,
            "solarsystem_id": 30000142,
            "solo": 0,
            "time": "2026-10-13T10:15:33.000000Z",
            "updated_at": "2026-10-13T10:16:33.000000Z",
            "value": 50000000.5
        }),
        120000003 => json!({
            "alliance_id": null,
            "character_id": 93813310,
            "corporation_id": 98000001,
            "created_at": "2026-10-12T10:16:33.000000Z",
            "damage_taken": 1500,
            "data": {
                "attackers": [
                    {
                        "alliance_id": null,
                        "character_id": 2112625428,
                        "corporation_id": 98000002,
                        "damage_done": 1000,
                        "faction_id": null,
                        "final_blow": true,
                        "security_status": -2.5,
                        "ship_type_id": 587,
                        "weapon_type_id": 3170
                    },
                    {
                        "alliance_id": null,
                        "character_id": 95465499,
                        "corporation_id": 98000003,
                        "damage_done": 500,
                        "faction_id": null,
                        "final_blow": false,
                        "security_status": 5,
                        "ship_type_id": 24698,
                        "weapon_type_id": 3170
                    }
                ],
                "killmail_id": 120000003,
                "killmail_time": "2026-10-12T10:15:33Z",
                "moon_id": null,
                "solar_system_id": 30002187,
                "victim": {
                    "alliance_id": null,
                    "character_id": 93813310,
                    "corporation_id": 98000001,
                    "damage_taken": 1500,
                    "faction_id": null,
                    "items": [],
                    "position": null,
                    "ship_type_id": 12044
                },
                "war_id": null
            },
            "hash": "hash120000003",
            "id": 120000003,
            "killmail_involvements": [
                {
                    "character_id": 2112625428,
                    "created_at": "2026-10-12T10:16:33.000000Z",
                    "id": 6,
                    "is_attacker": true,
                    "killmail_id": 120000003,
                    "updated_at": "2026-10-12T10:16:33.000000Z"
                },
                {
                    "character_id": 95465499,
                    "created_at": "2026-10-12T10:16:33.000000Z",
                    "id": 7,
                    "is_attacker": true,
                    "killmail_id": 120000003,
                    "updated_at": "2026-10-12T10:16:33.000000Z"
                },
                {
                    "character_id": 93813310,
                    "created_at": "2026-10-12T10:16:33.000000Z",
                    "id": 8,
                    "is_attacker": false,
                    "killmail_id": 120000003,
                    "updated_at": "2026-10-12T10:16:33.000000Z"
                }
            ],
            "ship_meta_group_id": null,
            "ship_type_id": 12044,
            "solarsystem_id": 30002187,
            "solo": 0,
            "time": "2026-10-12T10:15:33.000000Z",
            "updated_at": "2026-10-12T10:16:33.000000Z",
            "value": 250000000
        }),
        120000005 => json!({
            "alliance_id": null,
            "character_id": 93813310,
            "corporation_id": 98000001,
            "created_at": "2026-10-13T12:06:33.000000Z",
            "damage_taken": 700,
            "data": {
                "attackers": [
                    {
                        "alliance_id": null,
                        "character_id": 95465499,
                        "corporation_id": 98000003,
                        "damage_done": 700,
                        "faction_id": null,
                        "final_blow": true,
                        "security_status": 5,
                        "ship_type_id": 24698,
                        "weapon_type_id": 3170
                    }
                ],
                "killmail_id": 120000005,
                "killmail_time": "2026-10-13T12:05:33Z",
                "moon_id": null,
                "solar_system_id": 30000142,
                "victim": {
                    "alliance_id": null,
                    "character_id": 93813310,
                    "corporation_id": 98000001,
                    "damage_taken": 700,
                    "faction_id": null,
                    "items": [],
                    "position": null,
                    "ship_type_id": 626
                },
                "war_id": null
            },
            "hash": "hash120000005",
            "id": 120000005,
            "killmail_involvements": [
                {
                    "character_id": 95465499,
                    "created_at": "2026-10-13T12:06:33.000000Z",
                    "id": 11,
                    "is_attacker": true,
                    "killmail_id": 120000005,
                    "updated_at": "2026-10-13T12:06:33.000000Z"
                },
                {
                    "character_id": 93813310,
                    "created_at": "2026-10-13T12:06:33.000000Z",
                    "id": 12,
                    "is_attacker": false,
                    "killmail_id": 120000005,
                    "updated_at": "2026-10-13T12:06:33.000000Z"
                }
            ],
            "ship_meta_group_id": 1,
            "ship_type_id": 626,
            "solarsystem_id": 30000142,
            "solo": 1,
            "time": "2026-10-13T12:05:33.000000Z",
            "updated_at": "2026-10-13T12:06:33.000000Z",
            "value": 3000000
        }),
        120000006 => json!({
            "alliance_id": null,
            "character_id": 93813310,
            "corporation_id": 98000001,
            "created_at": "2026-10-13T09:16:33.000000Z",
            "damage_taken": 52000,
            "data": {
                "attackers": [
                    {
                        "alliance_id": null,
                        "character_id": 2112625428,
                        "corporation_id": 98000002,
                        "damage_done": 52000,
                        "faction_id": null,
                        "final_blow": true,
                        "security_status": -2.5,
                        "ship_type_id": 587,
                        "weapon_type_id": 3170
                    }
                ],
                "killmail_id": 120000006,
                "killmail_time": "2026-10-13T09:15:33Z",
                "moon_id": null,
                "solar_system_id": 30002187,
                "victim": {
                    "alliance_id": null,
                    "character_id": 93813310,
                    "corporation_id": 98000001,
                    "damage_taken": 52000,
                    "faction_id": null,
                    "items": [],
                    "position": null,
                    "ship_type_id": 29984
                },
                "war_id": 12345
            },
            "hash": "hash120000006",
            "id": 120000006,
            "killmail_involvements": [
                {
                    "character_id": 2112625428,
                    "created_at": "2026-10-13T09:16:33.000000Z",
                    "id": 13,
                    "is_attacker": true,
                    "killmail_id": 120000006,
                    "updated_at": "2026-10-13T09:16:33.000000Z"
                },
                {
                    "character_id": 93813310,
                    "created_at": "2026-10-13T09:16:33.000000Z",
                    "id": 14,
                    "is_attacker": false,
                    "killmail_id": 120000006,
                    "updated_at": "2026-10-13T09:16:33.000000Z"
                }
            ],
            "ship_meta_group_id": 14,
            "ship_type_id": 29984,
            "solarsystem_id": 30002187,
            "solo": 0,
            "time": "2026-10-13T09:15:33.000000Z",
            "updated_at": "2026-10-13T09:16:33.000000Z",
            "value": null
        }),
        other => panic!("{other} is not a dataset killmail"),
    }
}

async fn page(app: &TestApp, path: &str) -> Value {
    app.get(&api_path(path)).await.ok_json()
}

/// What an overlay page answered and left behind.
struct Overlay {
    path: &'static str,
    character: i64,
    named: Option<&'static str>,
    options: Value,
    killmails: &'static [i64],
    jobs: Value,
    requests: &'static [&'static str],
}

async fn check_overlay(overlay: Overlay) {
    let app = TestApp::with_dataset().await;

    let props = page(&app, overlay.path).await;

    let named: Vec<(i64, &str)> = overlay
        .named
        .map(|name| (overlay.character, name))
        .into_iter()
        .collect();
    let characters = characters(&[overlay.character], &named);
    let mut expected = overlay.options;
    expected["character"] = character_row(&characters, overlay.character);
    expected["killmails"] = overlay.killmails.iter().map(|id| killmail(*id)).collect();
    assert_json_eq(&expected, &props);
    assert_json_eq(&characters, &app.character_rows().await);
    assert_json_eq(&overlay.jobs, &queued(&app).await);
    assert_eq!(app.eve.paths(), overlay.requests, "the outgoing requests");
}

#[tokio::test]
async fn home() {
    let app = TestApp::with_dataset().await;

    assert_json_eq(
        &json!({
            "discord_invite_url": "https://discord.gg/d2NZ5DRDQe",
            "max_killmail_age_hours": 168,
            "max_visible_killmails": 50,
            "sim_ships": [
                {
                    "meta_group_id": 1,
                    "type_id": 587
                },
                {
                    "meta_group_id": 1,
                    "type_id": 593
                },
                {
                    "meta_group_id": 1,
                    "type_id": 602
                },
                {
                    "meta_group_id": 1,
                    "type_id": 626
                },
                {
                    "meta_group_id": 1,
                    "type_id": 638
                },
                {
                    "meta_group_id": 1,
                    "type_id": 645
                },
                {
                    "meta_group_id": 2,
                    "type_id": 12044
                },
                {
                    "meta_group_id": 4,
                    "type_id": 17738
                },
                {
                    "meta_group_id": 1,
                    "type_id": 24698
                },
                {
                    "meta_group_id": 14,
                    "type_id": 29984
                },
                {
                    "meta_group_id": 14,
                    "type_id": 29990
                },
                {
                    "meta_group_id": 14,
                    "type_id": 34562
                },
                {
                    "meta_group_id": 1,
                    "type_id": 671
                }
            ]
        }),
        &page(&app, "/").await,
    );
    assert_json_eq(&characters(&[], &[]), &app.character_rows().await);
    assert_eq!(app.count("jobs").await, 0);
    assert_eq!(app.eve.paths(), SIM_SHIP_LOOKUPS);
}

/// The meta group lookups of the simulation's ships, which neither the
/// dataset nor the mock knows.
const SIM_SHIP_LOOKUPS: [&str; 13] = [
    "/esi/universe/types/587",
    "/esi/universe/types/593",
    "/esi/universe/types/602",
    "/esi/universe/types/626",
    "/esi/universe/types/638",
    "/esi/universe/types/645",
    "/esi/universe/types/12044",
    "/esi/universe/types/17738",
    "/esi/universe/types/24698",
    "/esi/universe/types/29984",
    "/esi/universe/types/29990",
    "/esi/universe/types/34562",
    "/esi/universe/types/671",
];

/// The demo's props: the options and the simulation's ships.
fn demo(count: i64, hours: i64, summary: bool, meta: bool) -> Value {
    json!({
        "count": count,
        "hours": hours,
        "summary": summary,
        "meta": meta,
        "sim_ships": [
            {
                "meta_group_id": 1,
                "type_id": 587
            },
            {
                "meta_group_id": 1,
                "type_id": 593
            },
            {
                "meta_group_id": 1,
                "type_id": 602
            },
            {
                "meta_group_id": 1,
                "type_id": 626
            },
            {
                "meta_group_id": 1,
                "type_id": 638
            },
            {
                "meta_group_id": 1,
                "type_id": 645
            },
            {
                "meta_group_id": 2,
                "type_id": 12044
            },
            {
                "meta_group_id": 4,
                "type_id": 17738
            },
            {
                "meta_group_id": 1,
                "type_id": 24698
            },
            {
                "meta_group_id": 14,
                "type_id": 29984
            },
            {
                "meta_group_id": 14,
                "type_id": 29990
            },
            {
                "meta_group_id": 14,
                "type_id": 34562
            },
            {
                "meta_group_id": 1,
                "type_id": 671
            }
        ]
    })
}

/// The options read from the query: the leading integer of a number,
/// the usual words for true and false, out-of-range values clamped to 1
/// through the maximums.
#[tokio::test]
async fn demo_options() {
    for (path, count, hours, summary, meta) in [
        ("/demo", 50, 168, true, true),
        ("/demo/", 50, 168, true, true),
        (
            "/demo?count=9999&hours=9999&summary=1&meta=0",
            50,
            168,
            true,
            false,
        ),
        ("/demo?count=10abc&hours=1.9", 10, 1, true, true),
        ("/demo?count=1e2&hours=-5", 50, 1, true, true),
        ("/demo?count=abc&hours=", 1, 1, true, true),
        ("/demo?count=&hours=0", 1, 1, true, true),
        ("/demo?count=%2012%20&hours=+7", 12, 7, true, true),
        ("/demo?count=007&hours=0x1A", 7, 1, true, true),
        (
            "/demo?count=99999999999999999999&hours=24",
            50,
            24,
            true,
            true,
        ),
        ("/demo?count[]=5&hours[]=", 1, 1, true, true),
        ("/demo?count=3&count=4", 4, 168, true, true),
        ("/demo?summary=yes&meta=off", 50, 168, true, false),
        ("/demo?summary=ON&meta=TRUE", 50, 168, true, true),
        ("/demo?summary=&meta=", 50, 168, false, false),
        ("/demo?summary=0&meta=1", 50, 168, false, true),
        ("/demo?summary=%20true%20&meta=no", 50, 168, true, false),
        ("/demo?summary=2&meta=y", 50, 168, false, false),
        ("/demo?summary[]=1", 50, 168, false, true),
        ("/demo?hours=1e1", 50, 10, true, true),
        ("/demo?hours=2.5e1", 50, 25, true, true),
    ] {
        let app = TestApp::with_dataset().await;
        assert_json_eq(&demo(count, hours, summary, meta), &page(&app, path).await);
        assert_json_eq(&characters(&[], &[]), &app.character_rows().await);
        assert_eq!(app.count("jobs").await, 0);
        assert_eq!(app.eve.paths(), SIM_SHIP_LOOKUPS, "{path}");
    }
}

/// The overlay with the default options: the killmails of the last week, newest first; zKillboard's latest killmails not stored yet are queued, and so is the meta group of a stored killmail without one.
#[tokio::test]
async fn character() {
    check_overlay(Overlay {
        path: "/2112625428",
        character: 2112625428,
        named: None,
        options: json!({"count": 50, "hours": 168, "summary": false, "meta": true, "killmails_count": 4, "kills_value": 251000000, "losses_value": 50000000.5}),
        killmails: &[120000006, 120000003, 120000002, 120000001],
        jobs: json!([backfill(120000099), backfill(120000098), meta_group(120000003)]),
        requests: &["/zkillboard/api/characterID/2112625428/"],
    })
    .await;
}

/// The cards stop at `count`; the totals still cover the whole window.
#[tokio::test]
async fn character_count_2() {
    check_overlay(Overlay {
        path: "/2112625428?count=2",
        character: 2112625428,
        named: None,
        options: json!({"count": 2, "hours": 168, "summary": false, "meta": true, "killmails_count": 4, "kills_value": 251000000, "losses_value": 50000000.5}),
        killmails: &[120000006, 120000003],
        jobs: json!([backfill(120000099), meta_group(120000003)]),
        requests: &["/zkillboard/api/characterID/2112625428/"],
    })
    .await;
}

/// Only the killmails of the last hour.
#[tokio::test]
async fn character_hours_1() {
    check_overlay(Overlay {
        path: "/2112625428?hours=1",
        character: 2112625428,
        named: None,
        options: json!({"count": 50, "hours": 1, "summary": false, "meta": true, "killmails_count": 1, "kills_value": 1000000, "losses_value": 0}),
        killmails: &[120000001],
        jobs: json!([backfill(120000099), backfill(120000098)]),
        requests: &["/zkillboard/api/characterID/2112625428/"],
    })
    .await;
}

/// Every option set at once.
#[tokio::test]
async fn character_options() {
    check_overlay(Overlay {
        path: "/2112625428?count=3&hours=25&summary=1&meta=0",
        character: 2112625428,
        named: None,
        options: json!({"count": 3, "hours": 25, "summary": true, "meta": false, "killmails_count": 3, "kills_value": 1000000, "losses_value": 50000000.5}),
        killmails: &[120000006, 120000002, 120000001],
        jobs: json!([backfill(120000099), backfill(120000098)]),
        requests: &["/zkillboard/api/characterID/2112625428/"],
    })
    .await;
}

/// Leading zeros in the id are the same character.
#[tokio::test]
async fn character_leading_zeros() {
    check_overlay(Overlay {
        path: "/02112625428",
        character: 2112625428,
        named: None,
        options: json!({"count": 50, "hours": 168, "summary": false, "meta": true, "killmails_count": 4, "kills_value": 251000000, "losses_value": 50000000.5}),
        killmails: &[120000006, 120000003, 120000002, 120000001],
        jobs: json!([backfill(120000099), backfill(120000098), meta_group(120000003)]),
        requests: &["/zkillboard/api/characterID/2112625428/"],
    })
    .await;
}

/// A trailing slash is the same page.
#[tokio::test]
async fn character_trailing_slash() {
    check_overlay(Overlay {
        path: "/2112625428/?count=1",
        character: 2112625428,
        named: None,
        options: json!({"count": 1, "hours": 168, "summary": false, "meta": true, "killmails_count": 4, "kills_value": 251000000, "losses_value": 50000000.5}),
        killmails: &[120000006],
        jobs: json!([backfill(120000099)]),
        requests: &["/zkillboard/api/characterID/2112625428/"],
    })
    .await;
}

/// A stored character without a name gets it from ESI first.
#[tokio::test]
async fn character_resolved() {
    check_overlay(Overlay {
        path: "/93813310",
        character: 93813310,
        named: Some("Victim Pilot"),
        options: json!({"count": 50, "hours": 168, "summary": false, "meta": true, "killmails_count": 5, "kills_value": 50000000.5, "losses_value": 254000000}),
        killmails: &[120000006, 120000005, 120000003, 120000002, 120000001],
        jobs: json!([meta_group(120000003)]),
        requests: &["/esi/characters/93813310", "/zkillboard/api/characterID/93813310/"],
    })
    .await;
}

/// A character that is only ever an attacker.
#[tokio::test]
async fn character_attacker() {
    check_overlay(Overlay {
        path: "/95465499?hours=1",
        character: 95465499,
        named: None,
        options: json!({"count": 50, "hours": 1, "summary": false, "meta": true, "killmails_count": 1, "kills_value": 3000000, "losses_value": 0}),
        killmails: &[120000005],
        jobs: json!([]),
        requests: &["/zkillboard/api/characterID/95465499/"],
    })
    .await;
}

/// zKillboard failing all five tries: the overlay renders from the stored killmails, and the character is still marked as listening.
#[tokio::test]
async fn character_zkillboard_down() {
    check_overlay(Overlay {
        path: "/90000001",
        character: 90000001,
        named: None,
        options: json!({"count": 50, "hours": 168, "summary": false, "meta": true, "killmails_count": 0, "kills_value": 0, "losses_value": 0}),
        killmails: &[],
        jobs: json!([]),
        requests: &["/zkillboard/api/characterID/90000001/", "/zkillboard/api/characterID/90000001/", "/zkillboard/api/characterID/90000001/", "/zkillboard/api/characterID/90000001/", "/zkillboard/api/characterID/90000001/"],
    })
    .await;
}

/// An id ESI does not know is a 404 and is not stored.
#[tokio::test]
async fn character_unknown() {
    let app = TestApp::with_dataset().await;

    let response = app.get(&api_path("/1")).await;

    assert_eq!(response.status, 404);
    assert_eq!(response.json(), json!({ "message": "" }));
    assert_json_eq(&characters(&[], &[]), &app.character_rows().await);
    assert_eq!(app.count("jobs").await, 0);
    assert_eq!(app.eve.paths(), ["/esi/characters/1"]);
}

/// An id beyond 64 bits is a 404 without asking anyone.
#[tokio::test]
async fn character_overflow() {
    let app = TestApp::with_dataset().await;

    let response = app.get(&api_path("/99999999999999999999")).await;

    assert_eq!(response.status, 404);
    assert_json_eq(&characters(&[], &[]), &app.character_rows().await);
    assert!(app.eve.received().is_empty());
}
