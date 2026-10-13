//! Reading an R2Z2 firehose item.

use serde_json::{Value, json};

use killfeed::killmails::dto::R2z2Killmail;

fn r2z2_payload() -> Value {
    json!({
        "killmail_id": 12345,
        "hash": "abc123hash",
        "esi": {
            "killmail_id": 12345,
            "killmail_time": "2024-01-15T12:00:00Z",
            "solar_system_id": 30002187,
            "victim": {
                "character_id": 93813310,
                "corporation_id": 98000001,
                "damage_taken": 500,
                "ship_type_id": 670,
                "items": []
            },
            "attackers": [{
                "character_id": 95465499,
                "corporation_id": 98000002,
                "damage_done": 500,
                "final_blow": true,
                "security_status": -2.5,
                "ship_type_id": 587,
                "weapon_type_id": 3170
            }]
        },
        "zkb": {
            "hash": "abc123hash",
            "locationID": 40009081,
            "fittedValue": 5000.25,
            "totalValue": 10000.50,
            "droppedValue": 3000.00,
            "destroyedValue": 7000.50,
            "points": 10,
            "npc": false,
            "solo": true,
            "awox": false,
            "labels": ["pvp"]
        },
        "uploaded_at": 1773950479,
        "sequence_id": 100
    })
}

#[test]
fn it_parses_an_r2z2_payload_into_killmail_id_hash_and_sequence() {
    let r2z2 = R2z2Killmail::parse(100, &r2z2_payload()).expect("parses");
    assert_eq!(r2z2.killmail_id, 12345);
    assert_eq!(r2z2.hash, "abc123hash");
    assert_eq!(r2z2.sequence_id, 100);
}

#[test]
fn it_converts_to_esi_killmail() {
    let esi = R2z2Killmail::parse(100, &r2z2_payload())
        .and_then(|r2z2| r2z2.to_esi())
        .expect("converts");
    assert_eq!(esi.killmail_id, 12345);
    assert_eq!(esi.solar_system_id, 30002187);
    assert_eq!(esi.victim.character_id, Some(93813310));
    assert_eq!(esi.victim.ship_type_id, Some(670));
    assert_eq!(esi.attackers.len(), 1);
    assert_eq!(esi.attackers[0].character_id, Some(95465499));
}

#[test]
fn it_converts_to_zkill_killmail() {
    let zkb = R2z2Killmail::parse(100, &r2z2_payload())
        .and_then(|r2z2| r2z2.to_zkill())
        .expect("converts");
    assert_eq!(zkb.hash, "abc123hash");
    assert_eq!(zkb.total_value, 10000.50);
    assert!(!zkb.npc);
    assert!(zkb.solo);
    assert_eq!(zkb.id, Some(12345));
}

/// An item without its keys is refused instead of crashing the
/// listener.
#[test]
fn an_item_without_its_keys_is_refused() {
    assert!(R2z2Killmail::parse(100, &json!({ "killmail_id": 1 })).is_err());
    let mut payload = r2z2_payload();
    payload["esi"]["victim"] = json!("broken");
    let r2z2 = R2z2Killmail::parse(100, &payload).expect("the envelope parses");
    assert!(r2z2.to_esi().is_err());
}
