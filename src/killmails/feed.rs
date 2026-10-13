//! What an overlay shows (`KillmailFeedService`): the latest killmails of
//! a character within the time window, newest first, at most `count`,
//! with their involvements; and the totals over the whole window (the
//! count behind the `+N` tile, ISK destroyed and lost).

use serde_json::{Value, json};
use time::{Duration, OffsetDateTime};

use crate::json::optional_timestamp;
use crate::options::OverlayOptions;
use crate::server::AppState;

#[derive(Debug, sqlx::FromRow)]
struct KillmailRow {
    id: i64,
    character_id: Option<i64>,
    hash: String,
    data: Value,
    time: OffsetDateTime,
    solarsystem_id: i64,
    corporation_id: Option<i64>,
    alliance_id: Option<i64>,
    ship_type_id: i64,
    ship_meta_group_id: Option<i16>,
    damage_taken: i64,
    value: Option<f64>,
    solo: bool,
    created_at: Option<OffsetDateTime>,
    updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
struct InvolvementRow {
    id: i64,
    killmail_id: i64,
    character_id: i64,
    is_attacker: bool,
    created_at: Option<OffsetDateTime>,
    updated_at: Option<OffsetDateTime>,
}

impl InvolvementRow {
    fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "killmail_id": self.killmail_id,
            "character_id": self.character_id,
            "is_attacker": self.is_attacker,
            "created_at": optional_timestamp(self.created_at),
            "updated_at": optional_timestamp(self.updated_at),
        })
    }
}

/// The latest killmails, as Eloquent serialized
/// `Killmail::with('killmailInvolvements')` (`solo` as MySQL's 0 or 1,
/// the involvements by id), and the ids of those without a meta group,
/// which heal in the background.
pub async fn latest(
    state: &AppState,
    character_id: i64,
    options: &OverlayOptions,
) -> sqlx::Result<(Value, Vec<i64>)> {
    let since = window_start(state.clock.now(), options.hours);
    let killmails: Vec<KillmailRow> = sqlx::query_as(
        "select k.id, k.character_id, k.hash, k.data, k.time, k.solarsystem_id, k.corporation_id,
                k.alliance_id, k.ship_type_id, k.ship_meta_group_id, k.damage_taken, k.value,
                k.solo, k.created_at, k.updated_at
         from killmails k
         where exists (select 1 from killmail_involvements i
                       where i.killmail_id = k.id and i.character_id = $1)
           and k.time > $2
         order by k.id desc
         limit $3",
    )
    .bind(character_id)
    .bind(since)
    .bind(options.count)
    .fetch_all(&state.pool)
    .await?;
    let ids: Vec<i64> = killmails.iter().map(|killmail| killmail.id).collect();
    let involvements: Vec<InvolvementRow> = sqlx::query_as(
        "select id, killmail_id, character_id, is_attacker, created_at, updated_at
         from killmail_involvements where killmail_id = any($1) order by id",
    )
    .bind(&ids)
    .fetch_all(&state.pool)
    .await?;
    let missing_meta_group = killmails
        .iter()
        .filter(|killmail| killmail.ship_meta_group_id.is_none())
        .map(|killmail| killmail.id)
        .collect();
    let rows = killmails
        .into_iter()
        .map(|killmail| {
            let own: Vec<Value> = involvements
                .iter()
                .filter(|involvement| involvement.killmail_id == killmail.id)
                .map(InvolvementRow::to_json)
                .collect();
            json!({
                "id": killmail.id,
                "character_id": killmail.character_id,
                "hash": killmail.hash,
                "data": killmail.data,
                "time": optional_timestamp(Some(killmail.time)),
                "solarsystem_id": killmail.solarsystem_id,
                "corporation_id": killmail.corporation_id,
                "alliance_id": killmail.alliance_id,
                "ship_type_id": killmail.ship_type_id,
                "ship_meta_group_id": killmail.ship_meta_group_id,
                "damage_taken": killmail.damage_taken,
                "value": killmail.value,
                "solo": i32::from(killmail.solo),
                "created_at": optional_timestamp(killmail.created_at),
                "updated_at": optional_timestamp(killmail.updated_at),
                "killmail_involvements": own,
            })
        })
        .collect();
    Ok((Value::Array(rows), missing_meta_group))
}

/// The totals over the window: every killmail of the character in it
/// (not capped by `count`), the value of its kills and of its losses (a
/// killmail without a value counts 0).
pub async fn stats(
    state: &AppState,
    character_id: i64,
    options: &OverlayOptions,
) -> sqlx::Result<Value> {
    let (count, kills, losses): (i64, f64, f64) = sqlx::query_as(
        "select count(distinct k.id),
                coalesce(sum(case when i.is_attacker then k.value else 0 end), 0)::float8,
                coalesce(sum(case when not i.is_attacker then k.value else 0 end), 0)::float8
         from killmails k
         join killmail_involvements i on i.killmail_id = k.id
         where i.character_id = $1 and k.time > $2",
    )
    .bind(character_id)
    .bind(window_start(state.clock.now(), options.hours))
    .fetch_one(&state.pool)
    .await?;
    Ok(json!({
        "killmails_count": count,
        "kills_value": kills,
        "losses_value": losses,
    }))
}

fn window_start(now: OffsetDateTime, hours: i64) -> OffsetDateTime {
    now - Duration::hours(hours)
}
