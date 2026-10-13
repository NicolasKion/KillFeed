//! The durable job queue.

use serde::{Deserialize, Serialize};

use crate::killmails::dto::{EsiKillmail, ZkillKillmail};
use crate::server::AppState;

pub const PROCESS_KILLMAIL: &str = "process_killmail";
pub const BACKFILL_META_GROUP: &str = "backfill_meta_group";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessKillmail {
    pub killmail_id: i64,
    pub hash: String,
    pub zkb: Option<ZkillKillmail>,
    pub esi: Option<EsiKillmail>,
}

pub type JobError = Box<dyn std::error::Error + Send + Sync>;

pub async fn enqueue_process_killmail(
    _state: &AppState,
    _job: &ProcessKillmail,
) -> sqlx::Result<bool> {
    todo!()
}

pub async fn enqueue_backfill_meta_group(
    _state: &AppState,
    _killmail_id: i64,
) -> sqlx::Result<bool> {
    todo!()
}

pub async fn process_killmail(_state: &AppState, _job: &ProcessKillmail) -> Result<(), JobError> {
    todo!()
}

pub async fn work_until_idle(_state: &AppState) {
    todo!()
}

pub async fn release_reserved(_pool: &sqlx::PgPool) -> sqlx::Result<u64> {
    todo!()
}
