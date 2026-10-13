//! The ESI client: unversioned routes with a compatibility date, and the
//! error limit door (FIXES.md, deliberate changes).

use time::Duration;

use crate::common::TestApp;
use crate::common::mock_eve::Reply;

#[tokio::test]
async fn requests_carry_the_compatibility_date() {
    let app = TestApp::new().await;
    app.eve.on(
        "/esi/characters/2112625428",
        Reply::json(serde_json::json!({ "name": "Capsuleer Prime" })),
    );

    app.get_json("/characters/2112625428/preview").await;

    let request = &app.eve.received()[0];
    assert_eq!(request.path, "/esi/characters/2112625428");
    assert_eq!(request.header("x-compatibility-date"), Some("2026-08-18"));
}

/// When ESI reports fewer than 10 errors left in its window, no call goes
/// out (it would risk the 420 ban for the whole server) until the window
/// resets; the callers see a failure.
#[tokio::test]
async fn calls_stop_when_the_error_limit_runs_low() {
    let app = TestApp::new().await;
    app.eve.on(
        "/esi/characters/1",
        Reply::status(404)
            .with_header("x-esi-error-limit-remain", "9")
            .with_header("x-esi-error-limit-reset", "30"),
    );
    app.eve.on(
        "/esi/characters/2112625428",
        Reply::json(serde_json::json!({ "name": "Capsuleer Prime" })),
    );

    assert_eq!(app.get_json("/characters/1/preview").await.status, 404);
    assert_eq!(
        app.get_json("/characters/2112625428/preview").await.status,
        404
    );
    assert_eq!(app.eve.received().len(), 1);

    app.clock.advance(Duration::seconds(31));
    assert_eq!(
        app.get_json("/characters/2112625428/preview").await.status,
        200
    );
}
