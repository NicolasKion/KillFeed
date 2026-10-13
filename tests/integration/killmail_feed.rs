//! tests/Feature/KillmailFeedTest.php: the overlay heals killmails stored
//! before meta groups were tracked.

use serde_json::json;

use crate::common::TestApp;
use crate::common::factories::{CharacterFactory, KillmailFactory, involve};
use crate::common::mock_eve::Reply;

#[tokio::test]
async fn killmails_stored_before_meta_groups_were_tracked_heal_when_served() {
    let app = TestApp::new().await;
    let now = app.clock.now();
    app.eve.on(
        "/zkillboard/api/characterID/90000100/",
        Reply::json(json!([])),
    );
    app.eve.esi_type(587, None, Some(2));

    let character = CharacterFactory {
        id: Some(90000100),
        ..CharacterFactory::default()
    }
    .create(&app.pool, now)
    .await;
    let killmail = KillmailFactory {
        ship_meta_group_id: None,
        ..KillmailFactory::default()
    }
    .create(&app.pool, now)
    .await;
    involve(&app.pool, killmail, character, true, now).await;

    assert_eq!(
        app.get(&format!("/api/pages/{character}")).await.status,
        200
    );
    app.work_jobs().await;

    let meta_group: Option<i16> =
        sqlx::query_scalar("select ship_meta_group_id from killmails where id = $1")
            .bind(killmail)
            .fetch_one(&app.pool)
            .await
            .expect("reads");
    assert_eq!(meta_group, Some(2));
}
