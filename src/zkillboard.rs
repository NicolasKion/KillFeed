//! The per-character backfill (`ZKillboardService`): every overlay view
//! asks zKillboard for the character's latest killmails and queues the
//! ones not stored yet, at most `count`. The listing is kept five
//! minutes, so the overlay's 10 second poll stays within zKillboard's
//! limits. This also heals any gap the firehose listener left (a restart,
//! the cutover).
//!
//! Five tries like `Http::retry(5)`; when zKillboard still fails, the
//! overlay renders from the stored killmails (FIXES.md: Laravel answered
//! 500). An empty listing is kept like any other, and entries that do not
//! parse are skipped (FIXES.md).

use serde_json::Value;
use time::Duration;

use crate::jobs::{ProcessKillmail, enqueue_process_killmail};
use crate::killmails::dto::ZkillKillmail;
use crate::server::AppState;

/// How long a listing is kept.
const KEEP_FOR: Duration = Duration::minutes(5);

/// Tries of a listing fetch.
const TRIES: usize = 5;

pub async fn queue_recent_killmails(state: &AppState, character_id: i64, count: i64) {
    let Some(listing) = listing(state, character_id).await else {
        return;
    };
    let entries = listing.as_array().map(Vec::as_slice).unwrap_or_default();
    let killmails = entries
        .iter()
        .filter_map(|entry| match ZkillKillmail::from_value(entry) {
            Ok(killmail) => Some(killmail),
            Err(error) => {
                tracing::warn!(character_id, %error, "zKillboard listing entry skipped");
                None
            }
        })
        .take(usize::try_from(count).unwrap_or(0));
    for killmail in killmails {
        let Some(killmail_id) = killmail.id else {
            continue;
        };
        let stored: Result<bool, sqlx::Error> =
            sqlx::query_scalar("select exists (select 1 from killmails where id = $1)")
                .bind(killmail_id)
                .fetch_one(&state.pool)
                .await;
        let queued = match stored {
            Ok(true) => continue,
            Ok(false) => {
                enqueue_process_killmail(
                    state,
                    &ProcessKillmail {
                        killmail_id,
                        hash: killmail.hash.clone(),
                        zkb: Some(killmail),
                        esi: None,
                    },
                )
                .await
            }
            Err(error) => Err(error),
        };
        if let Err(error) = queued {
            tracing::error!(character_id, killmail_id, %error, "killmail not queued");
        }
    }
}

/// The kept listing, or a fresh one from zKillboard (then kept).
async fn listing(state: &AppState, character_id: i64) -> Option<Value> {
    let now = state.clock.now();
    let kept: Result<Option<Value>, sqlx::Error> = sqlx::query_scalar(
        "select body from zkillboard_listings where character_id = $1 and expires_at > $2",
    )
    .bind(character_id)
    .bind(now)
    .fetch_optional(&state.pool)
    .await;
    match kept {
        Ok(Some(body)) => return Some(body),
        Ok(None) => {}
        Err(error) => tracing::error!(%error, character_id, "zKillboard listing not read"),
    }

    let path = format!("/api/characterID/{character_id}/");
    let mut body = None;
    for attempt in 1..=TRIES {
        match state.eve.zkillboard_get(&path).await {
            Ok(answer) if answer.ok() => match answer.json() {
                Ok(json) => {
                    body = Some(json);
                    break;
                }
                Err(error) => {
                    tracing::debug!(character_id, %error, attempt, "zKillboard answer unreadable")
                }
            },
            Ok(answer) => tracing::debug!(
                character_id,
                status = answer.status,
                attempt,
                "zKillboard failed"
            ),
            Err(error) => tracing::debug!(character_id, %error, attempt, "zKillboard failed"),
        }
    }
    let Some(body) = body else {
        tracing::warn!(
            character_id,
            "zKillboard listing failed, the overlay shows what is stored"
        );
        return None;
    };
    let stored = sqlx::query(
        "insert into zkillboard_listings (character_id, body, expires_at) values ($1, $2, $3)
         on conflict (character_id) do update set body = excluded.body, expires_at = excluded.expires_at",
    )
    .bind(character_id)
    .bind(&body)
    .bind(now + KEEP_FOR)
    .execute(&state.pool)
    .await;
    if let Err(error) = stored {
        tracing::error!(%error, character_id, "zKillboard listing not kept");
    }
    Some(body)
}
