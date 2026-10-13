//! The schedule.

use time::OffsetDateTime;

use crate::server::AppState;

pub fn next_midnight(_now: OffsetDateTime) -> OffsetDateTime {
    todo!()
}

pub async fn housekeeping(_state: &AppState) {
    todo!()
}
