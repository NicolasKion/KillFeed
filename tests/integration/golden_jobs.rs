//! The queue against `tests/fixtures/golden/jobs.json`: an overlay view
//! queues the zKillboard backfill (killmails not stored yet) and the meta
//! group heal (stored ones without it); the workers then store the
//! killmails as Laravel did (`data` with every key, the involvements, the
//! characters) and keep the one ESI never answered as failed.

use crate::common::golden::{self, involvements_by_id, job_kind, read_json, requests, written_now};
use crate::common::{TestApp, assert_json_eq};

#[tokio::test]
async fn the_overlay_backfill_stores_killmails_like_laravel() {
    let recorded = read_json("golden/jobs.json");
    let now = golden::recorded_now();
    let app = TestApp::with_dataset().await;

    let response = app
        .get(&golden::api_path(
            recorded["path"].as_str().expect("a path"),
        ))
        .await;
    assert_eq!(response.status, 200);

    let expected: Vec<(String, i64)> = recorded["queued"]
        .as_array()
        .expect("queued")
        .iter()
        .map(|job| {
            (
                job_kind(job["job"].as_str().expect("a class")).to_owned(),
                job["killmail_id"].as_i64().expect("an id"),
            )
        })
        .collect();
    assert_eq!(expected, app.queued_jobs().await);

    app.work_jobs().await;

    assert_eq!(requests(&recorded["requests"]), app.eve.paths());
    assert_json_eq(
        &involvements_by_id(&written_now(&recorded["killmails"], now)),
        &involvements_by_id(&written_now(&app.killmail_rows().await, now)),
    );
    assert_json_eq(
        &written_now(&recorded["characters"], now),
        &written_now(&app.character_rows().await, now),
    );
    assert_eq!(app.queued_jobs().await, Vec::<(String, i64)>::new());
    let failed: Vec<(String, i64)> = sqlx::query_as(
        "select kind, killmail_id from jobs where failed_at is not null order by id",
    )
    .fetch_all(&app.pool)
    .await
    .expect("reads");
    let expected_failed: Vec<(String, i64)> = recorded["failed_jobs"]
        .as_array()
        .expect("failed jobs")
        .iter()
        .map(|job| {
            (
                job_kind(job["job"].as_str().expect("a class")).to_owned(),
                job["killmail_id"].as_i64().expect("an id"),
            )
        })
        .collect();
    assert_eq!(expected_failed, failed);
}
