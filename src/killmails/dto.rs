//! The killmail shapes.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EsiKillmail {
    pub killmail_id: i64,
    pub killmail_time: String,
    pub moon_id: Option<i64>,
    pub solar_system_id: i64,
    pub victim: EsiVictim,
    pub attackers: Vec<EsiAttacker>,
    pub war_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EsiVictim {
    pub damage_taken: i64,
    pub items: Vec<EsiItem>,
    pub alliance_id: Option<i64>,
    pub character_id: Option<i64>,
    pub corporation_id: Option<i64>,
    pub faction_id: Option<i64>,
    pub position: Option<EsiPosition>,
    pub ship_type_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EsiAttacker {
    pub alliance_id: Option<i64>,
    pub character_id: Option<i64>,
    pub corporation_id: Option<i64>,
    pub damage_done: i64,
    pub faction_id: Option<i64>,
    pub final_blow: bool,
    pub security_status: f64,
    pub ship_type_id: Option<i64>,
    pub weapon_type_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EsiItem {
    pub flag: i64,
    pub item_type_id: i64,
    pub singleton: i64,
    pub quantity_destroyed: Option<i64>,
    pub quantity_dropped: Option<i64>,
    pub items: Vec<EsiItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EsiPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

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

#[derive(Debug, Clone)]
pub struct R2z2Killmail {
    pub sequence_id: i64,
    pub killmail_id: i64,
    pub hash: String,
    pub data: Value,
}

impl EsiKillmail {
    pub fn from_value(_value: &Value) -> Result<Self, String> {
        todo!()
    }
}

impl ZkillKillmail {
    pub fn from_value(_value: &Value) -> Result<Self, String> {
        todo!()
    }
}

impl R2z2Killmail {
    pub fn parse(_sequence_id: i64, _value: &Value) -> Result<Self, String> {
        todo!()
    }

    pub fn to_esi(&self) -> Result<EsiKillmail, String> {
        todo!()
    }

    pub fn to_zkill(&self) -> Result<ZkillKillmail, String> {
        todo!()
    }
}
