//! The per-character backfill from zKillboard on every overlay view
//! (`ZKillboardService`), which the PHP suite only faked: the listing is
//! kept five minutes, only killmails not stored yet are queued, at most
//! `count` of them, and zKillboard failing never breaks the overlay.

use serde_json::{Value, json};
use time::Duration;

use crate::common::TestApp;
use crate::common::factories::{CharacterFactory, KillmailFactory};
use crate::common::mock_eve::Reply;

const LISTING: &str = "/zkillboard/api/characterID/90000200/";

fn entry(id: i64) -> Value {
    json!({ "killmail_id": id, "zkb": {
        "locationID": 40009081, "hash": format!("zkb{id}"), "fittedValue": 10.0,
        "droppedValue": 1.0, "destroyedValue": 9.0, "totalValue": 10.0, "points": 1,
        "npc": false, "solo": false, "awox": false, "labels": []
    }})
}

async fn app_with_character() -> TestApp {
    let app = TestApp::new().await;
    let now = app.clock.now();
    CharacterFactory {
        id: Some(90000200),
        ..CharacterFactory::default()
    }
    .create(&app.pool, now)
    .await;
    app
}

#[tokio::test]
async fn only_killmails_not_stored_yet_are_queued_at_most_count() {
    let app = app_with_character().await;
    let now = app.clock.now();
    KillmailFactory {
        id: Some(702),
        ..KillmailFactory::default()
    }
    .create(&app.pool, now)
    .await;
    app.eve.on(
        LISTING,
        Reply::json(json!([entry(701), entry(702), entry(703), entry(704)])),
    );

    assert_eq!(app.get("/api/pages/90000200?count=3").await.status, 200);

    assert_eq!(
        app.queued_jobs().await,
        vec![
            ("process_killmail".to_owned(), 701),
            ("process_killmail".to_owned(), 703)
        ]
    );
    let payload: Value = sqlx::query_scalar("select payload from jobs where killmail_id = 701")
        .fetch_one(&app.pool)
        .await
        .expect("reads");
    assert_eq!(payload["hash"], "zkb701");
    assert_eq!(payload["zkb"]["total_value"], 10.0);
    assert_eq!(payload["esi"], Value::Null);
}

#[tokio::test]
async fn the_listing_is_kept_for_five_minutes() {
    let app = app_with_character().await;
    app.eve.on(LISTING, Reply::json(json!([entry(801)])));

    app.get("/api/pages/90000200").await;
    app.clock
        .advance(Duration::minutes(5) - Duration::seconds(1));
    app.get("/api/pages/90000200").await;
    assert_eq!(app.eve.paths_under("/zkillboard").len(), 1);

    app.clock.advance(Duration::seconds(2));
    app.get("/api/pages/90000200").await;
    assert_eq!(app.eve.paths_under("/zkillboard").len(), 2);
}

/// FIXES.md: an empty listing is kept like any other.
#[tokio::test]
async fn an_empty_listing_is_cached_too() {
    let app = app_with_character().await;
    app.eve.on(LISTING, Reply::json(json!([])));

    app.get("/api/pages/90000200").await;
    app.get("/api/pages/90000200").await;

    assert_eq!(app.eve.paths_under("/zkillboard").len(), 1);
}

/// FIXES.md: zKillboard failing (five tries) leaves the overlay rendering
/// from the stored killmails; nothing is kept, the next view asks again.
#[tokio::test]
async fn a_failing_zkillboard_still_renders_the_overlay() {
    let app = app_with_character().await;
    app.eve.on(LISTING, Reply::status(502));

    let response = app.get("/api/pages/90000200").await;

    assert_eq!(response.ok_json()["character"]["name"], "Test Pilot");
    assert_eq!(app.eve.paths_under("/zkillboard").len(), 5);
    assert_eq!(app.count("zkillboard_listings").await, 0);

    app.get("/api/pages/90000200").await;
    assert_eq!(app.eve.paths_under("/zkillboard").len(), 10);
}

/// FIXES.md: entries that do not parse are skipped; an error object
/// instead of a list queues nothing.
#[tokio::test]
async fn a_broken_listing_entry_is_skipped() {
    let app = app_with_character().await;
    app.eve.on(
        LISTING,
        Reply::json(json!([{ "killmail_id": 901 }, entry(902)])),
    );
    assert_eq!(app.get("/api/pages/90000200").await.status, 200);
    assert_eq!(
        app.queued_jobs().await,
        vec![("process_killmail".to_owned(), 902)]
    );

    let app = app_with_character().await;
    app.eve
        .on(LISTING, Reply::json(json!({ "error": "rate limited" })));
    assert_eq!(app.get("/api/pages/90000200").await.status, 200);
    assert!(app.queued_jobs().await.is_empty());
}
