//! The overlay page's own behavior beyond the golden props: its side
//! effects and the window and count rules (tests/Browser/CharacterPageTest
//! "marks the character as being listened to" is ported in Playwright as
//! well).

use serde_json::json;
use time::Duration;

use crate::common::TestApp;
use crate::common::factories::{CharacterFactory, KillmailFactory, involve};
use crate::common::mock_eve::Reply;

#[tokio::test]
async fn the_overlay_marks_the_character_as_listening() {
    let app = TestApp::new().await;
    let now = app.clock.now();
    let character = CharacterFactory::default()
        .create(&app.pool, now - Duration::hours(1))
        .await;
    app.eve.on(
        &format!("/zkillboard/api/characterID/{character}/"),
        Reply::json(json!([])),
    );

    let props = app.get(&format!("/api/pages/{character}")).await.ok_json();

    assert_eq!(
        props["character"]["listening_since"],
        "2026-10-12T14:30:00.000000Z"
    );
    assert_eq!(
        props["character"]["updated_at"],
        "2026-10-12T14:30:00.000000Z"
    );
    let listening: Option<time::OffsetDateTime> =
        sqlx::query_scalar("select listening_since from characters where id = $1")
            .bind(character)
            .fetch_one(&app.pool)
            .await
            .expect("reads");
    assert_eq!(listening, Some(now));
}

/// The cards stop at `count`; the count and the ISK totals cover every
/// killmail in the window (the overlay's `+N` tile).
#[tokio::test]
async fn the_totals_cover_the_whole_window() {
    let app = TestApp::new().await;
    let now = app.clock.now();
    let character = CharacterFactory::default().create(&app.pool, now).await;
    app.eve.on(
        &format!("/zkillboard/api/characterID/{character}/"),
        Reply::json(json!([])),
    );
    for (minutes, value, attacker) in [
        (10, Some(100.0), true),
        (20, Some(50.5), false),
        (30, None, true),
        (61, Some(7.0), true),
    ] {
        let killmail = KillmailFactory {
            minutes_ago: minutes,
            value,
            ..KillmailFactory::default()
        }
        .create(&app.pool, now)
        .await;
        involve(&app.pool, killmail, character, attacker, now).await;
    }

    let props = app
        .get(&format!("/api/pages/{character}?count=1&hours=1"))
        .await
        .ok_json();

    assert_eq!(props["killmails"].as_array().map(Vec::len), Some(1));
    assert_eq!(props["killmails_count"], 3);
    assert_eq!(props["kills_value"], 100);
    assert_eq!(props["losses_value"], 50.5);
}

#[tokio::test]
async fn an_unknown_character_is_not_found_and_not_stored() {
    let app = TestApp::new().await;

    let response = app.get("/api/pages/1").await;

    assert_eq!(response.status, 404);
    assert_eq!(app.count("characters").await, 0);
    assert_eq!(app.eve.paths(), vec!["/esi/characters/1"]);
}
