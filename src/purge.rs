//! The nightly purge of old killmails.

use sqlx::PgPool;
use time::OffsetDateTime;

pub async fn purge_old_killmails(
    _pool: &PgPool,
    _now: OffsetDateTime,
    _hours: i64,
) -> sqlx::Result<u64> {
    todo!()
}
