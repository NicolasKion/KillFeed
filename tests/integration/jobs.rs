//! The durable queue, which the PHP suite never covered: one pending job
//! per kind and killmail (Laravel's ShouldBeUnique), failed jobs kept,
//! the downtime, and jobs a stopped api had taken.

use time::macros::datetime;

use killfeed::jobs::{ProcessKillmail, enqueue_backfill_meta_group, enqueue_process_killmail};

use crate::common::TestApp;
use crate::common::mock_eve::Reply;

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

/// One try, as Laravel's workers ran them; the failed job is kept with
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

/// FIXES.md: a killmail job taken during EVE's downtime (11:00 to 11:15
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

/// The meta group heal is no ESI killmail fetch; it runs in downtime too,
/// as Laravel's had no downtime middleware.
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
