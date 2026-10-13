//! The schedule: the nightly purge (`app:purge-old-killmails` at 00:00
//! UTC) and the hourly housekeeping.

use serde_json::json;
use time::Duration;
use time::macros::datetime;

use killfeed::purge::purge_old_killmails;
use killfeed::scheduler::{housekeeping, next_midnight};

use crate::common::TestApp;
use crate::common::factories::{CharacterFactory, KillmailFactory, involve};

#[test]
fn the_purge_runs_at_the_next_midnight_in_utc() {
    assert_eq!(
        next_midnight(datetime!(2026-10-12 23:59:59 UTC)),
        datetime!(2026-10-13 00:00:00 UTC)
    );
    assert_eq!(
        next_midnight(datetime!(2026-10-13 00:00:00 UTC)),
        datetime!(2026-10-14 00:00:00 UTC)
    );
}

/// Killmails older than KILLMAIL_MAX_AGE_HOURS go with their
/// involvements; the characters stay.
#[tokio::test]
async fn the_purge_deletes_old_killmails_and_keeps_characters() {
    let app = TestApp::new().await;
    let now = app.clock.now();
    let character = CharacterFactory::default().create(&app.pool, now).await;
    let old = KillmailFactory {
        minutes_ago: 168 * 60 + 1,
        ..KillmailFactory::default()
    }
    .create(&app.pool, now)
    .await;
    let edge = KillmailFactory {
        minutes_ago: 168 * 60,
        ..KillmailFactory::default()
    }
    .create(&app.pool, now)
    .await;
    involve(&app.pool, old, character, true, now).await;
    involve(&app.pool, edge, character, true, now).await;

    let deleted = purge_old_killmails(&app.pool, now, 168)
        .await
        .expect("purges");

    assert_eq!(deleted, 1);
    let left: Vec<i64> = sqlx::query_scalar("select id from killmails")
        .fetch_all(&app.pool)
        .await
        .expect("reads");
    assert_eq!(left, vec![edge]);
    assert_eq!(app.count("killmail_involvements").await, 1);
    assert_eq!(app.count("characters").await, 1);
}

#[tokio::test]
async fn housekeeping_drops_expired_listings_and_old_failed_jobs() {
    let app = TestApp::new().await;
    let now = app.clock.now();
    sqlx::query(
        "insert into zkillboard_listings (character_id, body, expires_at)
         values (1, '[]', $1), (2, '[]', $2)",
    )
    .bind(now - Duration::seconds(1))
    .bind(now + Duration::minutes(1))
    .execute(&app.pool)
    .await
    .expect("inserts");
    sqlx::query(
        "insert into jobs (kind, killmail_id, payload, available_at, failed_at, created_at)
         values ('process_killmail', 1, $3, $1, $1, $1),
                ('process_killmail', 2, $3, $2, $2, $2),
                ('process_killmail', 3, $3, $1, null, $1)",
    )
    .bind(now - Duration::days(8))
    .bind(now - Duration::days(6))
    .bind(json!({}))
    .execute(&app.pool)
    .await
    .expect("inserts");

    housekeeping(&app.state).await;

    let listings: Vec<i64> = sqlx::query_scalar("select character_id from zkillboard_listings")
        .fetch_all(&app.pool)
        .await
        .expect("reads");
    assert_eq!(listings, vec![2]);
    let jobs: Vec<i64> = sqlx::query_scalar("select killmail_id from jobs order by killmail_id")
        .fetch_all(&app.pool)
        .await
        .expect("reads");
    assert_eq!(jobs, vec![2, 3]);
}
