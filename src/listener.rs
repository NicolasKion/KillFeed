//! The R2Z2 firehose listener (`app:listen-for-kills`): reads zKillboard's
//! killmail stream one sequence file at a time and queues the killmails
//! of characters with an overlay open (`listening_since` in the last five
//! minutes) with the zKillboard and ESI data the item carries, so no ESI
//! fetch is needed.
//!
//! - It starts after the sequence stored in `listener_state` (written
//!   after every item), or at `sequence.json` the first time.
//! - A missing sequence (404) is waited for 6 seconds at a time; after
//!   `R2Z2_MAX_CONSECUTIVE_404S` in a row it asks `sequence.json` where
//!   the firehose is and goes on from there. At the head of the stream
//!   that is usually a few sequences back, so killmails are read again;
//!   the queue's one-job-per-killmail rule and the storage make that a
//!   no-op (FIXES.md, needs owner decision).
//! - Any other failure, a connection error included, is retried on the
//!   same sequence after 6 seconds (Laravel's command died on a
//!   connection error and supervisor restarted it). An item that does not
//!   parse is logged and skipped (FIXES.md).
//! - On shutdown it finishes the current item, stores the sequence and
//!   ends (`app:stop-listening` is gone).
//!
//! R2Z2 keeps a sequence file for at least 24 hours and allows 15
//! requests a second per IP (zKillboard's R2Z2 wiki page); the 100 ms
//! between items keeps the listener under that. A gap (the listener
//! stopped, the cutover) is replayed in order while R2Z2 still has the
//! files, so any stop shorter than a day loses nothing; files it no longer has answer 404 like the
//! head of the stream, so after the ten 404s the listener jumps to the
//! present, and the gap heals through the per-character zKillboard
//! backfill of every overlay view (`zkillboard.rs`).

use std::collections::HashSet;

use serde_json::Value;
use time::{Duration, OffsetDateTime};
use tokio_util::sync::CancellationToken;

use crate::eve::Answer;
use crate::jobs::{ProcessKillmail, enqueue_process_killmail};
use crate::killmails::dto::R2z2Killmail;
use crate::server::AppState;

/// The row of `listener_state`.
const STATE_NAME: &str = "r2z2";

/// How long the watched characters are kept before they are read again.
const WATCHED_FOR: Duration = Duration::seconds(30);

/// How recent an overlay view keeps a character watched.
const LISTENING_WINDOW: Duration = Duration::minutes(5);

/// The characters to watch, read at most every 30 seconds.
struct Watched {
    ids: HashSet<i64>,
    read_at: Option<OffsetDateTime>,
}

impl Watched {
    async fn get(&mut self, state: &AppState) -> &HashSet<i64> {
        let now = state.clock.now();
        if self
            .read_at
            .is_none_or(|read_at| now - read_at >= WATCHED_FOR)
        {
            let ids: Result<Vec<i64>, sqlx::Error> =
                sqlx::query_scalar("select id from characters where listening_since > $1")
                    .bind(now - LISTENING_WINDOW)
                    .fetch_all(&state.pool)
                    .await;
            match ids {
                Ok(ids) => {
                    self.ids = ids.into_iter().collect();
                    self.read_at = Some(now);
                }
                Err(error) => tracing::error!(%error, "watched characters not read"),
            }
        }
        &self.ids
    }
}

pub async fn run(state: AppState, cancel: CancellationToken) {
    let timing = state.config.eve.timing.clone();
    let max_404s = state.config.eve.r2z2_max_consecutive_404s.max(1);

    let mut sequence = loop {
        match start_sequence(&state).await {
            Some(sequence) => break sequence,
            None => {
                tracing::error!("R2Z2: no sequence to start from, retrying");
                if pause(&cancel, timing.listener_retry_pause).await {
                    return;
                }
            }
        }
    };
    tracing::info!(sequence, "R2Z2: listening");

    let mut watched = Watched {
        ids: HashSet::new(),
        read_at: None,
    };
    let mut consecutive_404s = 0;
    while !cancel.is_cancelled() {
        let answer = state.eve.r2z2_get(&format!("/{sequence}.json")).await;
        match answer {
            Ok(Answer { status: 404, .. }) => {
                consecutive_404s += 1;
                tracing::debug!(
                    sequence,
                    consecutive_404s,
                    "R2Z2: sequence not available yet"
                );
                if consecutive_404s >= max_404s {
                    tracing::warn!(consecutive_404s, "R2Z2: asking where the firehose is");
                    if let Some(current) = current_sequence(&state).await {
                        tracing::info!(from = sequence, to = current, "R2Z2: skipped ahead");
                        sequence = current;
                        consecutive_404s = 0;
                        continue;
                    }
                }
                if pause(&cancel, timing.listener_retry_pause).await {
                    break;
                }
            }
            Ok(answer) if !answer.ok() => {
                tracing::error!(
                    sequence,
                    status = answer.status,
                    "R2Z2: request failed, retrying"
                );
                if pause(&cancel, timing.listener_retry_pause).await {
                    break;
                }
            }
            Err(error) => {
                tracing::error!(sequence, %error, "R2Z2: request failed, retrying");
                if pause(&cancel, timing.listener_retry_pause).await {
                    break;
                }
            }
            Ok(answer) => {
                consecutive_404s = 0;
                handle(&state, sequence, &answer, &mut watched).await;
                store_sequence(&state, sequence).await;
                sequence += 1;
                if pause(&cancel, timing.listener_item_pause).await {
                    break;
                }
            }
        }
    }
    tracing::info!(sequence, "R2Z2: stopped");
}

/// Sleeps, or returns true at once when the shutdown came first.
async fn pause(cancel: &CancellationToken, duration: std::time::Duration) -> bool {
    tokio::select! {
        () = cancel.cancelled() => true,
        () = tokio::time::sleep(duration) => false,
    }
}

/// One firehose item: queue it when a watched character is on it.
async fn handle(state: &AppState, sequence: i64, answer: &Answer, watched: &mut Watched) {
    let parsed = answer
        .json()
        .map_err(|error| error.to_string())
        .and_then(|body: Value| R2z2Killmail::parse(sequence, &body))
        .and_then(|item| Ok((item.to_esi()?, item.to_zkill()?, item)));
    let (esi, zkb, item) = match parsed {
        Ok(parsed) => parsed,
        Err(error) => {
            tracing::error!(sequence, %error, "R2Z2: item skipped");
            return;
        }
    };
    tracing::info!(
        killmail_id = item.killmail_id,
        sequence,
        victim_character_id = esi.victim.character_id,
        victim_ship_type_id = esi.victim.ship_type_id,
        solar_system_id = esi.solar_system_id,
        attacker_count = esi.attackers.len(),
        killmail_time = %esi.killmail_time,
        "R2Z2: received killmail"
    );
    let watched = watched.get(state).await;
    if watched.is_empty() || !esi.involved_character_ids().any(|id| watched.contains(&id)) {
        return;
    }
    tracing::info!(
        killmail_id = item.killmail_id,
        "R2Z2: queuing killmail of a watched character"
    );
    let job = ProcessKillmail {
        killmail_id: item.killmail_id,
        hash: item.hash,
        zkb: Some(zkb),
        esi: Some(esi),
    };
    if let Err(error) = enqueue_process_killmail(state, &job).await {
        tracing::error!(killmail_id = job.killmail_id, %error, "R2Z2: killmail not queued");
    }
}

/// After the stored sequence, else where the firehose is now.
async fn start_sequence(state: &AppState) -> Option<i64> {
    let stored: Result<Option<i64>, sqlx::Error> =
        sqlx::query_scalar("select sequence from listener_state where name = $1")
            .bind(STATE_NAME)
            .fetch_optional(&state.pool)
            .await;
    match stored {
        Ok(Some(sequence)) => {
            tracing::info!(sequence, "R2Z2: resuming after the stored sequence");
            Some(sequence + 1)
        }
        Ok(None) => current_sequence(state).await,
        Err(error) => {
            tracing::error!(%error, "R2Z2: stored sequence not read");
            None
        }
    }
}

/// `sequence.json`: the firehose's current sequence.
async fn current_sequence(state: &AppState) -> Option<i64> {
    match state.eve.r2z2_get("/sequence.json").await {
        Ok(answer) if answer.ok() => answer.json().ok()?["sequence"].as_i64(),
        Ok(answer) => {
            tracing::warn!(status = answer.status, "R2Z2: sequence.json failed");
            None
        }
        Err(error) => {
            tracing::warn!(%error, "R2Z2: sequence.json failed");
            None
        }
    }
}

async fn store_sequence(state: &AppState, sequence: i64) {
    let stored = sqlx::query(
        "insert into listener_state (name, sequence, updated_at) values ($1, $2, $3)
         on conflict (name) do update set sequence = excluded.sequence, updated_at = excluded.updated_at",
    )
    .bind(STATE_NAME)
    .bind(sequence)
    .bind(state.clock.now())
    .execute(&state.pool)
    .await;
    if let Err(error) = stored {
        tracing::error!(sequence, %error, "R2Z2: sequence not stored");
    }
}
