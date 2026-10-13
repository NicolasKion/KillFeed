//! Storing a killmail, with the ESI
//! data the firehose carried or fetched from ESI, plus how the storage
//! treats repeats and odd killmails.

use serde_json::{Value, json};

use killfeed::jobs::{ProcessKillmail, process_killmail};
use killfeed::killmails::dto::{EsiKillmail, ZkillKillmail};

use crate::common::mock_eve::Reply;
use crate::common::{TestApp, assert_json_eq};

fn esi(killmail_id: i64, damage: i64, security_status: f64) -> Value {
    json!({
        "killmail_id": killmail_id,
        "killmail_time": "2024-01-15T12:00:00Z",
        "moon_id": null,
        "solar_system_id": 30002187,
        "victim": {
            "character_id": 93813310,
            "corporation_id": 98000001,
            "alliance_id": null,
            "faction_id": null,
            "damage_taken": damage,
            "ship_type_id": 670,
            "items": []
        },
        "attackers": [{
            "character_id": 95465499,
            "corporation_id": 98000002,
            "alliance_id": null,
            "faction_id": null,
            "damage_done": damage,
            "final_blow": true,
            "security_status": security_status,
            "ship_type_id": 587,
            "weapon_type_id": 3170
        }]
    })
}

fn zkb(killmail_id: i64, hash: &str, total: f64, solo: bool) -> ZkillKillmail {
    ZkillKillmail::from_value(&json!({
        "killmail_id": killmail_id,
        "zkb": {
            "hash": hash,
            "locationID": 40009081,
            "fittedValue": total / 2.0,
            "totalValue": total,
            "droppedValue": 3000.0,
            "destroyedValue": 7000.0,
            "points": 10,
            "npc": false,
            "solo": solo,
            "awox": false,
            "labels": []
        }
    }))
    .expect("zKillboard data parses")
}

async fn stored(app: &TestApp, id: i64) -> Option<(String, Option<i16>, Option<f64>, bool)> {
    sqlx::query_as("select hash, ship_meta_group_id, value, solo from killmails where id = $1")
        .bind(id)
        .fetch_optional(&app.pool)
        .await
        .expect("reads")
}

#[tokio::test]
async fn it_stores_a_killmail_when_full_data_is_provided_without_fetching_it() {
    let app = TestApp::new().await;
    app.eve.esi_type(670, Some(2), None);
    let job = ProcessKillmail {
        killmail_id: 99999,
        hash: "testhash123".to_owned(),
        zkb: Some(zkb(99999, "testhash123", 10000.0, true)),
        esi: Some(EsiKillmail::from_value(&esi(99999, 500, -2.5)).expect("parses")),
    };

    process_killmail(&app.state, &job).await.expect("stores");

    assert!(
        app.eve
            .paths_under("/esi")
            .iter()
            .all(|path| !path.contains("/killmails/"))
    );
    let (hash, meta_group, value, solo) = stored(&app, 99999).await.expect("stored");
    assert_eq!(hash, "testhash123");
    assert_eq!(meta_group, Some(2));
    assert_eq!(value, Some(10000.0));
    assert!(solo);
}

#[tokio::test]
async fn it_fetches_esi_killmail_when_esi_killmail_is_null() {
    let app = TestApp::new().await;
    app.eve.esi_type(670, None, None);
    app.eve.on(
        "/esi/killmails/88888/fallbackhash",
        Reply::json(esi(88888, 300, 1.0)),
    );
    let job = ProcessKillmail {
        killmail_id: 88888,
        hash: "fallbackhash".to_owned(),
        zkb: Some(zkb(88888, "fallbackhash", 2000.0, false)),
        esi: None,
    };

    process_killmail(&app.state, &job).await.expect("stores");

    assert_eq!(app.eve.received().len(), 2);
    let (_, meta_group, _, _) = stored(&app, 88888).await.expect("stored");
    assert_eq!(meta_group, Some(1));
}

#[tokio::test]
async fn it_returns_early_when_zkill_killmail_is_null() {
    let app = TestApp::new().await;
    let job = ProcessKillmail {
        killmail_id: 77777,
        hash: "somehash".to_owned(),
        zkb: None,
        esi: None,
    };

    process_killmail(&app.state, &job)
        .await
        .expect("does nothing");

    assert!(app.eve.received().is_empty());
    assert!(stored(&app, 77777).await.is_none());
}

/// Five tries a second apart, then the job fails.
#[tokio::test]
async fn a_killmail_esi_does_not_answer_fails_after_five_tries() {
    let app = TestApp::new().await;
    app.eve.on("/esi/killmails/66666/gone", Reply::status(502));
    let job = ProcessKillmail {
        killmail_id: 66666,
        hash: "gone".to_owned(),
        zkb: Some(zkb(66666, "gone", 1.0, false)),
        esi: None,
    };

    assert!(process_killmail(&app.state, &job).await.is_err());
    assert_eq!(app.eve.paths(), vec!["/esi/killmails/66666/gone"; 5]);
    assert!(stored(&app, 66666).await.is_none());
}

/// The killmail as stored: the characters (attackers need a corporation),
/// the columns, the involvements (attackers first, then the victim), and
/// `data` with every optional key present.
#[tokio::test]
async fn the_stored_killmail_keeps_every_key() {
    let app = TestApp::new().await;
    app.eve.esi_type(670, None, None);
    let mut raw = esi(55555, 800, 0.0);
    raw["attackers"]
        .as_array_mut()
        .expect("attackers")
        .push(json!({
            "character_id": 96000001, "damage_done": 0, "final_blow": false,
            "security_status": 1.1, "ship_type_id": 670
        }));
    let job = ProcessKillmail {
        killmail_id: 55555,
        hash: "h".to_owned(),
        zkb: Some(zkb(55555, "h", 1.5, false)),
        esi: Some(EsiKillmail::from_value(&raw).expect("parses")),
    };

    process_killmail(&app.state, &job).await.expect("stores");

    let data: Value = sqlx::query_scalar("select data from killmails where id = 55555")
        .fetch_one(&app.pool)
        .await
        .expect("reads");
    assert_json_eq(
        &json!({
            "killmail_id": 55555,
            "killmail_time": "2024-01-15T12:00:00Z",
            "moon_id": null,
            "solar_system_id": 30002187,
            "victim": {
                "damage_taken": 800, "items": [], "alliance_id": null,
                "character_id": 93813310, "corporation_id": 98000001,
                "faction_id": null, "position": null, "ship_type_id": 670
            },
            "attackers": [
                {
                    "alliance_id": null, "character_id": 95465499, "corporation_id": 98000002,
                    "damage_done": 800, "faction_id": null, "final_blow": true,
                    "security_status": 0.0, "ship_type_id": 587, "weapon_type_id": 3170
                },
                {
                    "alliance_id": null, "character_id": 96000001, "corporation_id": null,
                    "damage_done": 0, "faction_id": null, "final_blow": false,
                    "security_status": 1.1, "ship_type_id": 670, "weapon_type_id": null
                }
            ],
            "war_id": null
        }),
        &data,
    );
    let characters: Vec<(i64, Option<String>)> =
        sqlx::query_as("select id, name from characters order by id")
            .fetch_all(&app.pool)
            .await
            .expect("reads");
    assert_eq!(characters, vec![(93813310, None), (95465499, None)]);
    let involvements: Vec<(i64, bool)> =
        sqlx::query_as("select character_id, is_attacker from killmail_involvements order by id")
            .fetch_all(&app.pool)
            .await
            .expect("reads");
    assert_eq!(involvements, vec![(95465499, true), (93813310, false)]);
}

/// Storing the same killmail again (the listener re-reads after its skip
/// ahead) changes nothing.
#[tokio::test]
async fn storing_a_killmail_twice_changes_nothing() {
    let app = TestApp::new().await;
    app.eve.esi_type(670, None, None);
    let job = ProcessKillmail {
        killmail_id: 44444,
        hash: "h".to_owned(),
        zkb: Some(zkb(44444, "h", 1.0, false)),
        esi: Some(EsiKillmail::from_value(&esi(44444, 1, 0.5)).expect("parses")),
    };
    process_killmail(&app.state, &job).await.expect("stores");
    app.clock.advance(time::Duration::hours(1));

    process_killmail(&app.state, &job)
        .await
        .expect("stores again");

    assert_eq!(app.count("killmails").await, 1);
    assert_eq!(app.count("killmail_involvements").await, 2);
    let changed: bool =
        sqlx::query_scalar("select updated_at <> created_at from killmails where id = 44444")
            .fetch_one(&app.pool)
            .await
            .expect("reads");
    assert!(!changed, "an unchanged killmail keeps its updated_at");
}

/// A character who is attacker and victim is stored once, as
/// the attacker, instead of failing the whole killmail.
#[tokio::test]
async fn a_character_on_both_sides_is_stored_once() {
    let app = TestApp::new().await;
    app.eve.esi_type(670, None, None);
    let mut raw = esi(33333, 10, 0.0);
    raw["attackers"][0]["character_id"] = json!(93813310);
    let job = ProcessKillmail {
        killmail_id: 33333,
        hash: "h".to_owned(),
        zkb: Some(zkb(33333, "h", 1.0, false)),
        esi: Some(EsiKillmail::from_value(&raw).expect("parses")),
    };

    process_killmail(&app.state, &job).await.expect("stores");

    let involvements: Vec<(i64, bool)> =
        sqlx::query_as("select character_id, is_attacker from killmail_involvements order by id")
            .fetch_all(&app.pool)
            .await
            .expect("reads");
    assert_eq!(involvements, vec![(93813310, true)]);
    assert_eq!(app.count("killmails").await, 1);
}
