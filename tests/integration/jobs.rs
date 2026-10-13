//! The durable queue: one pending job per kind and killmail, failed jobs
//! kept, the downtime, jobs a stopped api had taken, and the backfill an
//! overlay view starts.

use serde_json::{Value, json};
use time::macros::datetime;

use killfeed::jobs::{ProcessKillmail, enqueue_backfill_meta_group, enqueue_process_killmail};

use crate::common::mock_eve::Reply;
use crate::common::{TestApp, assert_json_eq};

fn without_data(killmail_id: i64) -> ProcessKillmail {
    ProcessKillmail {
        killmail_id,
        hash: "h".to_owned(),
        zkb: None,
        esi: None,
    }
}

#[tokio::test]
async fn a_killmail_is_queued_once_while_its_job_is_pending() {
    let app = TestApp::new().await;

    assert!(
        enqueue_process_killmail(&app.state, &without_data(1))
            .await
            .expect("queues")
    );
    assert!(
        !enqueue_process_killmail(&app.state, &without_data(1))
            .await
            .expect("queues")
    );
    assert!(
        enqueue_backfill_meta_group(&app.state, 1)
            .await
            .expect("queues")
    );
    assert!(
        !enqueue_backfill_meta_group(&app.state, 1)
            .await
            .expect("queues")
    );

    assert_eq!(
        app.queued_jobs().await,
        vec![
            ("process_killmail".to_owned(), 1),
            ("backfill_meta_group".to_owned(), 1)
        ]
    );
}

#[tokio::test]
async fn a_done_job_is_deleted_and_can_be_queued_again() {
    let app = TestApp::new().await;
    enqueue_process_killmail(&app.state, &without_data(2))
        .await
        .expect("queues");

    app.work_jobs().await;

    assert_eq!(app.count("jobs").await, 0);
    assert!(
        enqueue_process_killmail(&app.state, &without_data(2))
            .await
            .expect("queues")
    );
}

/// One try; the failed job is kept with
/// its error and does not block the killmail from being queued again.
#[tokio::test]
async fn a_failed_job_is_kept_and_does_not_block_a_new_one() {
    let app = TestApp::new().await;
    app.eve.on("/esi/killmails/3/h", Reply::status(500));
    let job = ProcessKillmail {
        zkb: Some(
            killfeed::killmails::dto::ZkillKillmail::from_value(&serde_json::json!({
                "killmail_id": 3,
                "zkb": { "hash": "h", "fittedValue": 0, "totalValue": 0, "droppedValue": 0,
                         "destroyedValue": 0, "points": 0, "npc": false, "solo": false,
                         "awox": false, "labels": [] }
            }))
            .expect("parses"),
        ),
        ..without_data(3)
    };
    enqueue_process_killmail(&app.state, &job)
        .await
        .expect("queues");

    app.work_jobs().await;

    let (attempts, error): (i32, Option<String>) = sqlx::query_as(
        "select attempts, last_error from jobs where killmail_id = 3 and failed_at is not null",
    )
    .fetch_one(&app.pool)
    .await
    .expect("the failed job is kept");
    assert_eq!(attempts, 1);
    assert!(error.is_some());
    assert!(
        enqueue_process_killmail(&app.state, &job)
            .await
            .expect("queues")
    );
}

/// A killmail job taken during EVE's downtime (11:00 to 11:15
/// UTC) waits until 11:15 without running.
#[tokio::test]
async fn a_killmail_job_waits_out_the_downtime() {
    let app = TestApp::at(datetime!(2026-10-12 11:05:00 UTC)).await;
    app.eve.on("/esi/killmails/4/h", Reply::status(500));
    enqueue_process_killmail(
        &app.state,
        &ProcessKillmail {
            zkb: Some(
                killfeed::killmails::dto::ZkillKillmail::from_value(&serde_json::json!({
                    "killmail_id": 4,
                    "zkb": { "hash": "h", "fittedValue": 0, "totalValue": 0, "droppedValue": 0,
                             "destroyedValue": 0, "points": 0, "npc": false, "solo": false,
                             "awox": false, "labels": [] }
                }))
                .expect("parses"),
            ),
            ..without_data(4)
        },
    )
    .await
    .expect("queues");

    app.work_jobs().await;

    assert!(app.eve.received().is_empty(), "nothing ran");
    let (available_at, attempts): (time::OffsetDateTime, i32) =
        sqlx::query_as("select available_at, attempts from jobs where killmail_id = 4")
            .fetch_one(&app.pool)
            .await
            .expect("still queued");
    assert_eq!(available_at, datetime!(2026-10-12 11:15:00 UTC));
    assert_eq!(attempts, 0);
}

#[tokio::test]
async fn a_killmail_job_runs_right_after_the_downtime() {
    let app = TestApp::at(datetime!(2026-10-12 11:05:00 UTC)).await;
    enqueue_process_killmail(&app.state, &without_data(5))
        .await
        .expect("queues");
    app.work_jobs().await;
    assert_eq!(app.count("jobs").await, 1);

    app.clock.set(datetime!(2026-10-12 11:15:00 UTC));
    app.work_jobs().await;

    assert_eq!(app.count("jobs").await, 0);
}

/// The meta group heal is no ESI killmail fetch; it runs in downtime too.
#[tokio::test]
async fn the_meta_group_heal_ignores_the_downtime() {
    let app = TestApp::at(datetime!(2026-10-12 11:05:00 UTC)).await;
    enqueue_backfill_meta_group(&app.state, 6)
        .await
        .expect("queues");

    app.work_jobs().await;

    assert_eq!(app.count("jobs").await, 0);
}

/// Jobs a stopped api had taken go back to the queue on the next start.
#[tokio::test]
async fn jobs_of_a_stopped_api_are_released() {
    let app = TestApp::new().await;
    enqueue_process_killmail(&app.state, &without_data(7))
        .await
        .expect("queues");
    sqlx::query("update jobs set reserved_at = now()")
        .execute(&app.pool)
        .await
        .expect("reserves");

    let released = killfeed::jobs::release_reserved(&app.pool)
        .await
        .expect("releases");
    app.work_jobs().await;

    assert_eq!(released, 1);
    assert_eq!(app.count("jobs").await, 0);
}

/// An overlay view queues zKillboard's killmails not stored yet and the
/// meta group of a stored one without it. The workers then store each
/// killmail with every key of its `data`, its involvements and its meta
/// group, heal the stored one, and keep the killmail ESI never answered
/// as a failed job after five tries.
#[tokio::test]
async fn the_overlay_backfill_stores_killmails() {
    let app = TestApp::with_dataset().await;
    let before = app.killmail_rows().await;

    let response = app.get("/api/pages/2112625428").await;
    assert_eq!(response.status, 200);
    assert_eq!(
        app.queued_jobs().await,
        [
            ("process_killmail".to_owned(), 120000099),
            ("process_killmail".to_owned(), 120000098),
            ("backfill_meta_group".to_owned(), 120000003),
        ]
    );

    app.work_jobs().await;

    assert_eq!(
        app.eve.paths(),
        [
            "/zkillboard/api/characterID/2112625428/",
            "/esi/killmails/120000099/zkbhash99",
            "/esi/universe/types/11176",
            "/esi/killmails/120000098/zkbhash98",
            "/esi/killmails/120000098/zkbhash98",
            "/esi/killmails/120000098/zkbhash98",
            "/esi/killmails/120000098/zkbhash98",
            "/esi/killmails/120000098/zkbhash98",
            "/esi/universe/types/12044",
        ]
    );
    let mut expected = before.as_array().expect("rows").clone();
    let healed = expected
        .iter_mut()
        .find(|row| row["id"] == 120000003)
        .expect("a dataset killmail");
    healed["ship_meta_group_id"] = json!(2);
    healed["updated_at"] = json!("2026-10-13T12:15:33.000000Z");
    expected.push(json!({
        "alliance_id": 99000002,
        "character_id": 93813310,
        "corporation_id": 98000001,
        "created_at": "2026-10-13T12:15:33.000000Z",
        "damage_taken": 3100,
        "data": {
            "attackers": [
                {
                    "alliance_id": 99000001,
                    "character_id": 2112625428,
                    "corporation_id": 98000002,
                    "damage_done": 3000,
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
                    "damage_done": 100,
                    "faction_id": null,
                    "final_blow": false,
                    "security_status": 0,
                    "ship_type_id": 24698,
                    "weapon_type_id": null
                },
                {
                    "alliance_id": null,
                    "character_id": 96000001,
                    "corporation_id": null,
                    "damage_done": 0,
                    "faction_id": null,
                    "final_blow": false,
                    "security_status": 1.1,
                    "ship_type_id": 670,
                    "weapon_type_id": null
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
            "killmail_id": 120000099,
            "killmail_time": "2026-10-13T12:10:33Z",
            "moon_id": null,
            "solar_system_id": 30002187,
            "victim": {
                "alliance_id": 99000002,
                "character_id": 93813310,
                "corporation_id": 98000001,
                "damage_taken": 3100,
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
                        "item_type_id": 3020,
                        "items": [
                            {
                                "flag": 5,
                                "item_type_id": 2205,
                                "items": [],
                                "quantity_destroyed": 3,
                                "quantity_dropped": null,
                                "singleton": 0
                            }
                        ],
                        "quantity_destroyed": null,
                        "quantity_dropped": 1,
                        "singleton": 0
                    }
                ],
                "position": {
                    "x": 1.5,
                    "y": -2.25,
                    "z": 300000000000_i64
                },
                "ship_type_id": 11176
            },
            "war_id": null
        },
        "hash": "zkbhash99",
        "id": 120000099,
        "killmail_involvements": [
            {
                "character_id": 2112625428,
                "created_at": "2026-10-13T12:15:33.000000Z",
                "id": 15,
                "is_attacker": true,
                "killmail_id": 120000099,
                "updated_at": "2026-10-13T12:15:33.000000Z"
            },
            {
                "character_id": 95465499,
                "created_at": "2026-10-13T12:15:33.000000Z",
                "id": 16,
                "is_attacker": true,
                "killmail_id": 120000099,
                "updated_at": "2026-10-13T12:15:33.000000Z"
            },
            {
                "character_id": 93813310,
                "created_at": "2026-10-13T12:15:33.000000Z",
                "id": 17,
                "is_attacker": false,
                "killmail_id": 120000099,
                "updated_at": "2026-10-13T12:15:33.000000Z"
            }
        ],
        "ship_meta_group_id": 2,
        "ship_type_id": 11176,
        "solarsystem_id": 30002187,
        "solo": 0,
        "time": "2026-10-13T12:10:33.000000Z",
        "updated_at": "2026-10-13T12:15:33.000000Z",
        "value": 12500000.75
    }));
    assert_json_eq(&Value::Array(expected), &app.killmail_rows().await);
    assert!(app.queued_jobs().await.is_empty());
    let failed: Vec<(String, i64)> = sqlx::query_as(
        "select kind, killmail_id from jobs where failed_at is not null order by id",
    )
    .fetch_all(&app.pool)
    .await
    .expect("reads");
    assert_eq!(failed, [("process_killmail".to_owned(), 120000098)]);
}
