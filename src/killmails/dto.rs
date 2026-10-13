//! The killmail shapes: ESI's killmail (stored as `killmails.data` with
//! every optional key present, `null` when ESI left it out, as the PHP
//! DTOs serialized it), zKillboard's `zkb` block, and an R2Z2 firehose
//! item carrying both.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// `GET /killmails/{id}/{hash}` (the DTO `EsiKillmail`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EsiKillmail {
    pub killmail_id: i64,
    pub killmail_time: String,
    #[serde(default)]
    pub moon_id: Option<i64>,
    pub solar_system_id: i64,
    pub victim: EsiVictim,
    pub attackers: Vec<EsiAttacker>,
    #[serde(default)]
    pub war_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EsiVictim {
    pub damage_taken: i64,
    pub items: Vec<EsiItem>,
    #[serde(default)]
    pub alliance_id: Option<i64>,
    #[serde(default)]
    pub character_id: Option<i64>,
    #[serde(default)]
    pub corporation_id: Option<i64>,
    #[serde(default)]
    pub faction_id: Option<i64>,
    #[serde(default)]
    pub position: Option<EsiPosition>,
    #[serde(default)]
    pub ship_type_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EsiAttacker {
    #[serde(default)]
    pub alliance_id: Option<i64>,
    #[serde(default)]
    pub character_id: Option<i64>,
    #[serde(default)]
    pub corporation_id: Option<i64>,
    pub damage_done: i64,
    #[serde(default)]
    pub faction_id: Option<i64>,
    pub final_blow: bool,
    pub security_status: f64,
    #[serde(default)]
    pub ship_type_id: Option<i64>,
    #[serde(default)]
    pub weapon_type_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EsiItem {
    pub flag: i64,
    pub item_type_id: i64,
    pub singleton: i64,
    #[serde(default)]
    pub quantity_destroyed: Option<i64>,
    #[serde(default)]
    pub quantity_dropped: Option<i64>,
    #[serde(default)]
    pub items: Vec<EsiItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EsiPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// zKillboard's data of a killmail (the DTO `zKillKillmail`), from a
/// character listing or a firehose item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZkillKillmail {
    pub id: Option<i64>,
    pub hash: String,
    pub location_id: i64,
    pub fitted_value: f64,
    pub total_value: f64,
    pub dropped_value: f64,
    pub destroyed_value: f64,
    pub points: i64,
    pub npc: bool,
    pub solo: bool,
    pub awox: bool,
    pub labels: Vec<Value>,
}

/// One item of the R2Z2 firehose: `{killmail_id, hash, esi, zkb}`.
#[derive(Debug, Clone)]
pub struct R2z2Killmail {
    pub sequence_id: i64,
    pub killmail_id: i64,
    pub hash: String,
    pub data: Value,
}

impl EsiKillmail {
    /// ESI's JSON; a missing required key or a wrong type is an error.
    pub fn from_value(value: &Value) -> Result<Self, String> {
        serde_json::from_value(value.clone()).map_err(|error| format!("ESI killmail: {error}"))
    }

    /// Every attacker and the victim with a character id.
    pub fn involved_character_ids(&self) -> impl Iterator<Item = i64> + '_ {
        self.attackers
            .iter()
            .filter_map(|attacker| attacker.character_id)
            .chain(self.victim.character_id)
    }
}

/// A float the way PHP's `(float)` cast read it: a number or a numeric
/// string.
fn float(value: &Value, key: &str) -> Result<f64, String> {
    match value.get(key) {
        Some(Value::Number(number)) => number.as_f64().ok_or_else(|| format!("zkb.{key}")),
        Some(Value::String(text)) => Ok(text.trim().parse().unwrap_or(0.0)),
        Some(Value::Bool(flag)) => Ok(f64::from(u8::from(*flag))),
        Some(Value::Null) => Ok(0.0),
        _ => Err(format!("zkb.{key} is missing")),
    }
}

fn required<'a>(value: &'a Value, key: &str) -> Result<&'a Value, String> {
    value
        .get(key)
        .filter(|value| !value.is_null())
        .ok_or_else(|| format!("zkb.{key} is missing"))
}

impl ZkillKillmail {
    /// `{killmail_id | killID, zkb: {...}}`, as zKillboard's listing and
    /// the firehose write it.
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let zkb = value.get("zkb").ok_or("zkb is missing")?;
        let id = value
            .get("killmail_id")
            .and_then(Value::as_i64)
            .or_else(|| value.get("killID").and_then(Value::as_i64));
        Ok(Self {
            id,
            hash: required(zkb, "hash")?
                .as_str()
                .ok_or("zkb.hash is no string")?
                .to_owned(),
            location_id: zkb.get("locationID").and_then(Value::as_i64).unwrap_or(0),
            fitted_value: float(zkb, "fittedValue")?,
            total_value: float(zkb, "totalValue")?,
            dropped_value: float(zkb, "droppedValue")?,
            destroyed_value: float(zkb, "destroyedValue")?,
            points: required(zkb, "points")?
                .as_i64()
                .ok_or("zkb.points is no integer")?,
            npc: required(zkb, "npc")?
                .as_bool()
                .ok_or("zkb.npc is no bool")?,
            solo: required(zkb, "solo")?
                .as_bool()
                .ok_or("zkb.solo is no bool")?,
            awox: required(zkb, "awox")?
                .as_bool()
                .ok_or("zkb.awox is no bool")?,
            labels: required(zkb, "labels")?
                .as_array()
                .ok_or("zkb.labels is no list")?
                .clone(),
        })
    }
}

impl R2z2Killmail {
    /// The item of `sequence_id`; it needs at least its killmail id and
    /// hash.
    pub fn parse(sequence_id: i64, value: &Value) -> Result<Self, String> {
        let killmail_id = value
            .get("killmail_id")
            .and_then(Value::as_i64)
            .ok_or("killmail_id is missing")?;
        let hash = value
            .get("hash")
            .and_then(Value::as_str)
            .ok_or("hash is missing")?
            .to_owned();
        Ok(Self {
            sequence_id,
            killmail_id,
            hash,
            data: value.clone(),
        })
    }

    pub fn to_esi(&self) -> Result<EsiKillmail, String> {
        EsiKillmail::from_value(self.data.get("esi").unwrap_or(&Value::Null))
    }

    pub fn to_zkill(&self) -> Result<ZkillKillmail, String> {
        ZkillKillmail::from_value(&json!({
            "killmail_id": self.data["killmail_id"],
            "zkb": self.data.get("zkb").cloned().unwrap_or(Value::Null),
        }))
    }
}
