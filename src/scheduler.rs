//! The schedule, in UTC: the purge of old killmails daily at 00:00. A
//! run missed while the api was down is skipped (the purge is by age, so
//! the next night catches up). The housekeeping every hour drops expired
//! zKillboard listings and failed jobs older than a week.

use std::time::Duration as StdDuration;

use time::{Duration, OffsetDateTime, Time};
use tokio_util::sync::CancellationToken;

use crate::purge::{purge_old_killmails, report};
use crate::server::AppState;

/// How long a failed job is kept for a look.
const FAILED_JOBS_KEPT_FOR: Duration = Duration::days(7);

/// The next 00:00 UTC after `now`.
pub fn next_midnight(now: OffsetDateTime) -> OffsetDateTime {
    now.replace_time(Time::MIDNIGHT) + Duration::days(1)
}

/// Drops expired zKillboard listings and failed jobs older than a week.
pub async fn housekeeping(state: &AppState) {
    let now = state.clock.now();
    if let Err(error) = sqlx::query("delete from zkillboard_listings where expires_at <= $1")
        .bind(now)
        .execute(&state.pool)
        .await
    {
        tracing::error!(%error, "expired zKillboard listings not deleted");
    }
    if let Err(error) = sqlx::query("delete from jobs where failed_at < $1")
        .bind(now - FAILED_JOBS_KEPT_FOR)
        .execute(&state.pool)
        .await
    {
        tracing::error!(%error, "old failed jobs not deleted");
    }
}

fn until(state: &AppState, moment: OffsetDateTime) -> StdDuration {
    StdDuration::from_millis((moment - state.clock.now()).whole_milliseconds().max(0) as u64)
}

pub fn spawn(state: &AppState, cancel: &CancellationToken) -> Vec<tokio::task::JoinHandle<()>> {
    let purge = {
        let state = state.clone();
        let cancel = cancel.clone();
        tokio::spawn(async move {
            loop {
                let at = next_midnight(state.clock.now());
                tokio::select! {
                    () = cancel.cancelled() => return,
                    () = tokio::time::sleep(until(&state, at)) => {}
                }
                let hours = state.config.max_killmail_age_hours;
                match purge_old_killmails(&state.pool, state.clock.now(), hours).await {
                    Ok(deleted) => tracing::info!("{}", report(deleted, hours)),
                    Err(error) => tracing::error!(%error, "old killmails not purged"),
                }
            }
        })
    };
    let housekeeping = {
        let state = state.clone();
        let cancel = cancel.clone();
        tokio::spawn(async move {
            let mut hourly = tokio::time::interval(StdDuration::from_secs(3600));
            loop {
                tokio::select! {
                    () = cancel.cancelled() => return,
                    _ = hourly.tick() => housekeeping(&state).await,
                }
            }
        })
    };
    vec![purge, housekeeping]
}
