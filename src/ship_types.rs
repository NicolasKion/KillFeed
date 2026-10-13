//! Ship meta groups.

use serde::Serialize;

use crate::server::AppState;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SimShip {
    pub type_id: i64,
    pub meta_group_id: i16,
}

pub async fn meta_group_id(_state: &AppState, _type_id: i64) -> i16 {
    todo!()
}

pub async fn demo_ships(_state: &AppState) -> Vec<SimShip> {
    todo!()
}
