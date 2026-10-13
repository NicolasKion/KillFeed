//! tests/Feature/ShipTypeServiceTest.php: a type's meta group from ESI's
//! dogma attributes, kept per type.

use serde_json::json;
use time::Duration;

use killfeed::ship_types::{SimShip, demo_ships, meta_group_id};

use crate::common::TestApp;
use crate::common::mock_eve::Reply;

#[tokio::test]
async fn it_resolves_faction_ships_from_the_meta_group_attribute() {
    let app = TestApp::new().await;
    app.eve.on(
        "/esi/universe/types/17738",
        Reply::json(json!({
            "type_id": 17738,
            "name": "Machariel",
            "dogma_attributes": [
                { "attribute_id": 422, "value": 1.0 },
                { "attribute_id": 1692, "value": 4.0 }
            ]
        })),
    );

    assert_eq!(meta_group_id(&app.state, 17738).await, 4);
}

#[tokio::test]
async fn it_resolves_tech_ii_and_tech_iii_hulls_from_the_tech_level_attribute() {
    // ESI omits the meta group attribute (1692) on Tech II/III hulls.
    let app = TestApp::new().await;
    app.eve.on(
        "/esi/universe/types/12044",
        Reply::json(json!({ "type_id": 12044, "name": "Enyo",
            "dogma_attributes": [{ "attribute_id": 422, "value": 2.0 }] })),
    );
    app.eve.on(
        "/esi/universe/types/29984",
        Reply::json(json!({ "type_id": 29984, "name": "Tengu",
            "dogma_attributes": [{ "attribute_id": 422, "value": 3.0 }] })),
    );

    assert_eq!(meta_group_id(&app.state, 12044).await, 2);
    assert_eq!(meta_group_id(&app.state, 29984).await, 14);
}

#[tokio::test]
async fn it_defaults_to_tech_i_when_no_relevant_attributes_are_present() {
    let app = TestApp::new().await;
    app.eve.on(
        "/esi/universe/types/587",
        Reply::json(
            json!({ "type_id": 587, "name": "Rifter", "dogma_attributes": [
            { "attribute_id": 422, "value": 1.0 },
            { "attribute_id": 633, "value": 0.0 }
        ] }),
        ),
    );

    assert_eq!(meta_group_id(&app.state, 587).await, 1);
}

#[tokio::test]
async fn it_caches_the_meta_group_per_type_id() {
    let app = TestApp::new().await;
    app.eve.on(
        "/esi/universe/types/29990",
        Reply::json(json!({ "type_id": 29990, "name": "Loki",
            "dogma_attributes": [{ "attribute_id": 422, "value": 3.0 }] })),
    );

    assert_eq!(meta_group_id(&app.state, 29990).await, 14);
    assert_eq!(meta_group_id(&app.state, 29990).await, 14);

    assert_eq!(app.eve.received().len(), 1);
}

#[tokio::test]
async fn it_falls_back_to_tech_i_without_caching_when_esi_fails() {
    let app = TestApp::new().await;
    app.eve.on("/esi/universe/types/999999", Reply::status(500));

    assert_eq!(meta_group_id(&app.state, 999999).await, 1);

    // Three tries (Laravel's `Http::retry(3, 500)`), nothing kept.
    assert_eq!(app.eve.received().len(), 3);
    assert_eq!(app.count("ship_types").await, 0);
}

/// The 30 day cache of Laravel's `Cache::put(..., now()->addDays(30))`.
#[tokio::test]
async fn the_meta_group_is_asked_again_after_30_days() {
    let app = TestApp::new().await;
    app.eve.esi_type(587, None, Some(1));
    meta_group_id(&app.state, 587).await;
    app.clock.advance(Duration::days(30) + Duration::seconds(1));

    meta_group_id(&app.state, 587).await;

    assert_eq!(app.eve.received().len(), 2);
}

/// FIXES.md: a failed lookup of the demo ships is not kept for a month.
/// While ESI fails, a view asks once (three tries), not for each ship.
#[tokio::test]
async fn a_failed_demo_ship_lookup_is_retried() {
    let app = TestApp::new().await;
    app.eve.fail_with(503);
    let ships = demo_ships(&app.state).await;
    assert!(ships.iter().all(|ship| ship.meta_group_id == 1));
    assert_eq!(ships.len(), 13);
    assert_eq!(app.eve.received().len(), 3);

    app.eve.recover();
    for type_id in killfeed::ship_types::DEMO_SHIP_TYPE_IDS {
        app.eve.esi_type(type_id, None, Some(1));
    }
    app.eve.esi_type(17738, Some(4), Some(1));
    let ships = demo_ships(&app.state).await;

    assert!(ships.contains(&SimShip {
        type_id: 17738,
        meta_group_id: 4,
    }));
}
