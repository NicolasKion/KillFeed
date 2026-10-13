//! The durable job queue in Postgres, in place of Laravel's database
//! queue and its ten workers: `process_killmail` (store a killmail, with
//! the zKillboard and ESI data it came with, fetching ESI's when missing)
//! and `backfill_meta_group` (heal a killmail stored without its ship's
//! meta group).
//!
//! One pending job per kind and killmail (a partial unique index, what
//! `ShouldBeUnique` amounted to): queuing a killmail that is already
//! queued is a no-op, which also absorbs the listener reading killmails
//! again after its skip ahead. Workers in the api claim due jobs with
//! `for update skip locked`. A job gets one try like Laravel's workers
//! gave it; done jobs are deleted, failed ones kept with their error (and
//! dropped after a week by the housekeeping) without blocking the
//! killmail from being queued again.
//!
//! EVE's daily downtime (11:00 to 11:15 UTC): a killmail job taken then
//! waits until 11:15 without running (FIXES.md).

use std::time::Duration as StdDuration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use time::{OffsetDateTime, Time};
use tokio_util::sync::CancellationToken;

use crate::killmails::dto::{EsiKillmail, ZkillKillmail};
use crate::killmails::store;
use crate::server::AppState;
use crate::ship_types::meta_group_id;

pub const PROCESS_KILLMAIL: &str = "process_killmail";
pub const BACKFILL_META_GROUP: &str = "backfill_meta_group";

/// Tries of a killmail fetch from ESI (Laravel's `Http::retry(5, 1000)`).
const ESI_KILLMAIL_TRIES: usize = 5;

const DOWNTIME_START: Time = time::macros::time!(11:00);
const DOWNTIME_END: Time = time::macros::time!(11:15);

/// The payload of `process_killmail`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessKillmail {
    pub killmail_id: i64,
    pub hash: String,
    pub zkb: Option<ZkillKillmail>,
    pub esi: Option<EsiKillmail>,
}

pub type JobError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, sqlx::FromRow)]
struct Job {
    id: i64,
    kind: String,
    killmail_id: i64,
    payload: sqlx::types::Json<Value>,
}

async fn enqueue(
    state: &AppState,
    kind: &str,
    killmail_id: i64,
    payload: Value,
) -> sqlx::Result<bool> {
    let now = state.clock.now();
    let queued = sqlx::query(
        "insert into jobs (kind, killmail_id, payload, available_at, created_at)
         values ($1, $2, $3, $4, $4)
         on conflict (kind, killmail_id) where failed_at is null do nothing",
    )
    .bind(kind)
    .bind(killmail_id)
    .bind(payload)
    .bind(now)
    .execute(&state.pool)
    .await?
    .rows_affected()
        == 1;
    if queued {
        state.jobs.notify_one();
    }
    Ok(queued)
}

/// Queues a killmail to be stored; false when it is queued already.
pub async fn enqueue_process_killmail(
    state: &AppState,
    job: &ProcessKillmail,
) -> sqlx::Result<bool> {
    let payload = serde_json::to_value(job).map_err(|error| sqlx::Error::Encode(error.into()))?;
    enqueue(state, PROCESS_KILLMAIL, job.killmail_id, payload).await
}

/// Queues the meta group heal of a stored killmail.
pub async fn enqueue_backfill_meta_group(state: &AppState, killmail_id: i64) -> sqlx::Result<bool> {
    enqueue(
        state,
        BACKFILL_META_GROUP,
        killmail_id,
        Value::Object(Default::default()),
    )
    .await
}

/// `ProcessKillmail::handle`: nothing without zKillboard data (there is
/// no hash or value to store), ESI's killmail when the job came without
/// it, then the storage.
pub async fn process_killmail(state: &AppState, job: &ProcessKillmail) -> Result<(), JobError> {
    let Some(zkb) = &job.zkb else {
        return Ok(());
    };
    let esi = match &job.esi {
        Some(esi) => esi.clone(),
        None => fetch_killmail(state, job.killmail_id, &job.hash).await?,
    };
    store::store(state, &esi, zkb).await?;
    Ok(())
}

async fn fetch_killmail(
    state: &AppState,
    killmail_id: i64,
    hash: &str,
) -> Result<EsiKillmail, JobError> {
    let path = format!("/killmails/{killmail_id}/{hash}");
    let mut last = String::new();
    for attempt in 1..=ESI_KILLMAIL_TRIES {
        match state.eve.esi_get(&path).await {
            Ok(answer) if answer.ok() => {
                return Ok(EsiKillmail::from_value(&answer.json()?)?);
            }
            Ok(answer) => last = format!("ESI answered {} for {path}", answer.status),
            Err(error) => last = format!("{error} for {path}"),
        }
        if attempt < ESI_KILLMAIL_TRIES {
            tokio::time::sleep(state.config.eve.timing.esi_killmail_retry).await;
        }
    }
    Err(last.into())
}

/// `BackfillShipMetaGroup::handle`.
async fn backfill_meta_group(state: &AppState, killmail_id: i64) -> Result<(), JobError> {
    let row: Option<(i64, Option<i16>)> =
        sqlx::query_as("select ship_type_id, ship_meta_group_id from killmails where id = $1")
            .bind(killmail_id)
            .fetch_optional(&state.pool)
            .await?;
    let Some((ship_type_id, None)) = row else {
        return Ok(());
    };
    let meta_group = meta_group_id(state, ship_type_id).await;
    sqlx::query(
        "update killmails set ship_meta_group_id = $2, updated_at = $3
         where id = $1 and ship_meta_group_id is null",
    )
    .bind(killmail_id)
    .bind(meta_group)
    .bind(state.clock.now())
    .execute(&state.pool)
    .await?;
    Ok(())
}

/// 11:15 UTC when `now` is in EVE's downtime, else `None`.
pub fn downtime_end(now: OffsetDateTime) -> Option<OffsetDateTime> {
    (now.time() >= DOWNTIME_START && now.time() < DOWNTIME_END)
        .then(|| now.replace_time(DOWNTIME_END))
}

/// Takes the next due job, if any.
async fn claim(pool: &PgPool, now: OffsetDateTime) -> sqlx::Result<Option<Job>> {
    sqlx::query_as(
        "update jobs set reserved_at = $1, attempts = attempts + 1
         where id = (
             select id from jobs
             where failed_at is null and reserved_at is null and available_at <= $1
             order by available_at, id
             for update skip locked
             limit 1
         )
         returning id, kind, killmail_id, payload",
    )
    .bind(now)
    .fetch_optional(pool)
    .await
}

/// Jobs a stopped api had taken are free again (one api process runs the
/// workers).
pub async fn release_reserved(pool: &PgPool) -> sqlx::Result<u64> {
    let result = sqlx::query("update jobs set reserved_at = null where reserved_at is not null")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

async fn run(state: &AppState, job: &Job) -> Result<(), JobError> {
    match job.kind.as_str() {
        PROCESS_KILLMAIL => {
            let payload: ProcessKillmail = serde_json::from_value(job.payload.0.clone())?;
            process_killmail(state, &payload).await
        }
        BACKFILL_META_GROUP => backfill_meta_group(state, job.killmail_id).await,
        other => Err(format!("unknown job kind {other}").into()),
    }
}

/// Runs one claimed job to its end: deferred past the downtime, done
/// (deleted) or failed (kept with its error).
async fn process(state: &AppState, job: Job) {
    let now = state.clock.now();
    if job.kind == PROCESS_KILLMAIL
        && let Some(end) = downtime_end(now)
    {
        let deferred = sqlx::query(
            "update jobs set reserved_at = null, attempts = attempts - 1, available_at = $2
             where id = $1",
        )
        .bind(job.id)
        .bind(end)
        .execute(&state.pool)
        .await;
        if let Err(error) = deferred {
            tracing::error!(job = job.id, %error, "job not deferred");
        }
        return;
    }
    let outcome = run(state, &job).await;
    let stored = match &outcome {
        Ok(()) => {
            sqlx::query("delete from jobs where id = $1")
                .bind(job.id)
                .execute(&state.pool)
                .await
        }
        Err(error) => {
            tracing::error!(job = job.id, kind = %job.kind, killmail = job.killmail_id, %error, "job failed");
            sqlx::query(
                "update jobs set reserved_at = null, failed_at = $2, last_error = $3 where id = $1",
            )
            .bind(job.id)
            .bind(state.clock.now())
            .bind(error.to_string())
            .execute(&state.pool)
            .await
        }
    };
    if let Err(error) = stored {
        tracing::error!(job = job.id, %error, "job state not stored");
    }
}

/// Works off every job that is due, one after the other, then returns.
pub async fn work_until_idle(state: &AppState) {
    loop {
        match claim(&state.pool, state.clock.now()).await {
            Ok(Some(job)) => process(state, job).await,
            Ok(None) => return,
            Err(error) => {
                tracing::error!(%error, "job not claimed");
                return;
            }
        }
    }
}

/// The workers: `JOB_WORKERS` loops claiming due jobs, woken by new jobs
/// and every second for the deferred ones, until shutdown (a job being
/// worked on is finished first).
pub fn spawn_workers(
    state: &AppState,
    cancel: &CancellationToken,
) -> Vec<tokio::task::JoinHandle<()>> {
    (0..state.config.background.job_workers)
        .map(|_| {
            let state = state.clone();
            let cancel = cancel.clone();
            tokio::spawn(async move {
                while !cancel.is_cancelled() {
                    match claim(&state.pool, state.clock.now()).await {
                        Ok(Some(job)) => process(&state, job).await,
                        Ok(None) => {
                            tokio::select! {
                                () = cancel.cancelled() => {}
                                () = state.jobs.notified() => {}
                                () = tokio::time::sleep(StdDuration::from_secs(1)) => {}
                            }
                        }
                        Err(error) => {
                            tracing::error!(%error, "job not claimed");
                            tokio::select! {
                                () = cancel.cancelled() => {}
                                () = tokio::time::sleep(StdDuration::from_secs(5)) => {}
                            }
                        }
                    }
                }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::downtime_end;

    #[test]
    fn the_downtime_is_eleven_to_a_quarter_past_utc() {
        assert_eq!(downtime_end(datetime!(2026-10-12 10:59:59 UTC)), None);
        assert_eq!(
            downtime_end(datetime!(2026-10-12 11:00:00 UTC)),
            Some(datetime!(2026-10-12 11:15:00 UTC))
        );
        assert_eq!(
            downtime_end(datetime!(2026-10-12 11:14:59 UTC)),
            Some(datetime!(2026-10-12 11:15:00 UTC))
        );
        assert_eq!(downtime_end(datetime!(2026-10-12 11:15:00 UTC)), None);
    }
}
