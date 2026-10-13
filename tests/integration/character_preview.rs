//! The preview the setup page
//! fetches while a character id is typed, plus its failures.

use serde_json::json;

use crate::common::mock_eve::Reply;
use crate::common::{TestApp, assert_json_eq};

#[tokio::test]
async fn character_preview_returns_resolved_character_data() {
    let app = TestApp::new().await;
    app.eve.on(
        "/esi/characters/2112625428",
        Reply::json(json!({ "name": "Capsuleer Prime" })),
    );

    let response = app.get_json("/characters/2112625428/preview").await;

    assert_json_eq(
        &json!({
            "id": 2112625428_i64,
            "name": "Capsuleer Prime",
            "portrait": "https://images.evetech.net/characters/2112625428/portrait?size=256",
        }),
        &response.ok_json(),
    );
    let name: Option<String> =
        sqlx::query_scalar("select name from characters where id = 2112625428")
            .fetch_one(&app.pool)
            .await
            .expect("the character is stored");
    assert_eq!(name.as_deref(), Some("Capsuleer Prime"));
}

#[tokio::test]
async fn character_preview_route_only_accepts_numeric_ids() {
    let app = TestApp::new().await;
    assert_eq!(
        app.get("/characters/not-a-number/preview").await.status,
        404
    );
    assert!(app.eve.received().is_empty());
}

/// A character with a stored name is answered without asking ESI.
#[tokio::test]
async fn a_known_character_is_not_asked_for_again() {
    let app = TestApp::new().await;
    sqlx::query("insert into characters (id, name) values (95465499, 'Attacker One')")
        .execute(&app.pool)
        .await
        .expect("inserts");

    let response = app.get_json("/characters/95465499/preview").await;

    assert_eq!(response.ok_json()["name"], "Attacker One");
    assert!(app.eve.received().is_empty());
}

/// An id ESI does not know is a 404, `{"message": ""}` for the setup
/// page's fetch.
#[tokio::test]
async fn an_unknown_character_is_not_found() {
    let app = TestApp::new().await;

    let response = app.get_json("/characters/1/preview").await;

    assert_eq!(response.status, 404);
    assert_json_eq(&json!({ "message": "" }), &response.json());
    assert_eq!(app.count("characters").await, 0);
}

/// ESI answering 5xx or 420 is a 404 as well.
#[tokio::test]
async fn a_failing_esi_is_not_found() {
    let app = TestApp::new().await;
    app.eve.on("/esi/characters/2112625428", Reply::status(503));

    assert_eq!(
        app.get_json("/characters/2112625428/preview").await.status,
        404
    );
}

/// An unreachable ESI is a 404, not a 500.
#[tokio::test]
async fn an_unreachable_esi_is_not_found() {
    let mut app = TestApp::new().await;
    app.reconfigure(|config| config.eve.esi_base_url = "http://127.0.0.1:1".to_owned());

    assert_eq!(
        app.get_json("/characters/2112625428/preview").await.status,
        404
    );
    assert_eq!(app.get("/api/pages/2112625428").await.status, 404);
}

/// An id beyond 64 bits is a 404 without asking ESI.
#[tokio::test]
async fn an_id_beyond_64_bits_is_not_found() {
    let app = TestApp::new().await;

    assert_eq!(
        app.get_json("/characters/99999999999999999999/preview")
            .await
            .status,
        404
    );
    assert_eq!(app.get("/api/pages/99999999999999999999").await.status, 404);
    assert!(app.eve.received().is_empty());
}

/// With the EVE services switched off (the dev stack) nothing is asked
/// and an unknown character is not found.
#[tokio::test]
async fn with_the_eve_services_off_nothing_is_asked() {
    let mut app = TestApp::new().await;
    app.reconfigure(|config| config.eve.enabled = false);
    app.eve.on(
        "/esi/characters/2112625428",
        Reply::json(json!({ "name": "Capsuleer Prime" })),
    );

    assert_eq!(
        app.get_json("/characters/2112625428/preview").await.status,
        404
    );
    assert!(app.eve.received().is_empty());
}
