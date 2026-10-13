//! `app:purge-old-killmails`: killmails older than the longest time
//! window an overlay can ask for are deleted with their involvements; the
//! characters stay.

use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

/// Deletes the killmails older than `hours`; the number deleted.
pub async fn purge_old_killmails(
    pool: &PgPool,
    now: OffsetDateTime,
    hours: i64,
) -> sqlx::Result<u64> {
    let result = sqlx::query("delete from killmails where time < $1")
        .bind(now - Duration::hours(hours))
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// The command's report line.
pub fn report(deleted: u64, hours: i64) -> String {
    format!("Deleted {deleted} killmails older than {hours} hours")
}
