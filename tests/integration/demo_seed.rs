//! The local preview's sample data (`seed_demo`): an overlay that shows
//! something without ESI or zKillboard, safe to seed again.

use killfeed::demo::{DEMO_CHARACTER_ID, seed};

use crate::common::TestApp;

#[tokio::test]
async fn the_sample_overlay_shows_a_week_of_killmails() {
    let mut app = TestApp::new().await;
    app.reconfigure(|config| config.eve.enabled = false);
    let now = app.clock.now();

    assert_eq!(seed(&app.pool, now).await.expect("seeds"), 12);
    assert_eq!(seed(&app.pool, now).await.expect("seeds again"), 12);

    let props = app
        .get(&format!("/api/pages/{DEMO_CHARACTER_ID}?summary=1"))
        .await
        .ok_json();
    assert_eq!(props["character"]["name"], "Capsuleer Prime");
    assert_eq!(props["killmails_count"], 12);
    assert_eq!(props["killmails"].as_array().map(Vec::len), Some(12));
    let home = app.get("/api/pages/home").await.ok_json();
    assert!(
        home["sim_ships"]
            .as_array()
            .expect("ships")
            .iter()
            .any(|ship| ship["meta_group_id"] == 4)
    );
    assert!(app.eve.received().is_empty());
}
