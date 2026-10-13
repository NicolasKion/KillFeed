//! Storing a killmail (`KillmailService::store`) in one transaction: the
//! characters (every attacker with a character and a corporation, and the
//! victim), the killmail with ESI's data and zKillboard's hash, value and
//! solo flag, and the involvements, attackers first. Storing it again
//! changes nothing that did not change.

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;

use super::dto::{EsiKillmail, ZkillKillmail};
use crate::server::AppState;
use crate::ship_types::meta_group_id;

#[derive(Debug)]
pub enum StoreError {
    /// A killmail the table cannot hold (no victim ship, no readable
    /// time).
    Invalid(String),
    Database(sqlx::Error),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Invalid(why) => write!(f, "invalid killmail: {why}"),
            StoreError::Database(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<sqlx::Error> for StoreError {
    fn from(error: sqlx::Error) -> Self {
        StoreError::Database(error)
    }
}

/// ESI's `killmail_time` (`2024-01-15T12:00:00Z`), as Carbon parsed it.
fn parse_time(text: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(text, &Rfc3339).ok().or_else(|| {
        time::PrimitiveDateTime::parse(
            text,
            format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
        )
        .ok()
        .map(time::PrimitiveDateTime::assume_utc)
    })
}

pub async fn store(
    state: &AppState,
    esi: &EsiKillmail,
    zkb: &ZkillKillmail,
) -> Result<(), StoreError> {
    let killmail_id = zkb.id.unwrap_or(esi.killmail_id);
    let ship_type_id = esi
        .victim
        .ship_type_id
        .ok_or_else(|| StoreError::Invalid("the victim has no ship".to_owned()))?;
    let time = parse_time(&esi.killmail_time)
        .ok_or_else(|| StoreError::Invalid(format!("time {:?}", esi.killmail_time)))?;
    let data = serde_json::to_value(esi).map_err(|error| StoreError::Invalid(error.to_string()))?;
    let meta_group = meta_group_id(state, ship_type_id).await;
    let now = state.clock.now();

    // Attackers count with a character and a corporation (NPCs and
    // structures have neither).
    let attackers: Vec<i64> = esi
        .attackers
        .iter()
        .filter(|attacker| attacker.corporation_id.is_some())
        .filter_map(|attacker| attacker.character_id)
        .collect();
    let mut characters: Vec<i64> = attackers.clone();
    characters.extend(esi.victim.character_id);
    characters.sort_unstable();
    characters.dedup();

    let mut transaction = state.pool.begin().await?;
    sqlx::query(
        "insert into characters (id, created_at, updated_at)
         select id, $2, $2 from unnest($1::bigint[]) as id
         on conflict (id) do nothing",
    )
    .bind(&characters)
    .bind(now)
    .execute(&mut *transaction)
    .await?;

    sqlx::query(
        "insert into killmails (id, hash, character_id, data, time, solarsystem_id,
            corporation_id, alliance_id, ship_type_id, ship_meta_group_id, damage_taken,
            value, solo, created_at, updated_at)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $14)
         on conflict (id) do update set
             hash = excluded.hash, character_id = excluded.character_id, data = excluded.data,
             time = excluded.time, solarsystem_id = excluded.solarsystem_id,
             corporation_id = excluded.corporation_id, alliance_id = excluded.alliance_id,
             ship_type_id = excluded.ship_type_id,
             ship_meta_group_id = excluded.ship_meta_group_id,
             damage_taken = excluded.damage_taken, value = excluded.value, solo = excluded.solo,
             updated_at = case
                 when (killmails.hash, killmails.character_id, killmails.data, killmails.time,
                       killmails.solarsystem_id, killmails.corporation_id, killmails.alliance_id,
                       killmails.ship_type_id, killmails.ship_meta_group_id,
                       killmails.damage_taken, killmails.value, killmails.solo)
                      is distinct from
                      (excluded.hash, excluded.character_id, excluded.data, excluded.time,
                       excluded.solarsystem_id, excluded.corporation_id, excluded.alliance_id,
                       excluded.ship_type_id, excluded.ship_meta_group_id,
                       excluded.damage_taken, excluded.value, excluded.solo)
                 then excluded.updated_at else killmails.updated_at end",
    )
    .bind(killmail_id)
    .bind(&zkb.hash)
    .bind(esi.victim.character_id)
    .bind(&data)
    .bind(time)
    .bind(esi.solar_system_id)
    .bind(esi.victim.corporation_id)
    .bind(esi.victim.alliance_id)
    .bind(ship_type_id)
    .bind(meta_group)
    .bind(esi.victim.damage_taken)
    .bind(zkb.total_value)
    .bind(zkb.solo)
    .bind(now)
    .execute(&mut *transaction)
    .await?;

    // Attackers first, then the victim. A character already on the
    // killmail keeps its first role, so one character on both sides
    // does not hit the unique index.
    let roles = attackers
        .iter()
        .map(|id| (*id, true))
        .chain(esi.victim.character_id.map(|id| (id, false)));
    for (character_id, is_attacker) in roles {
        sqlx::query(
            "insert into killmail_involvements (killmail_id, character_id, is_attacker, created_at, updated_at)
             values ($1, $2, $3, $4, $4)
             on conflict (killmail_id, character_id) do nothing",
        )
        .bind(killmail_id)
        .bind(character_id)
        .bind(is_attacker)
        .bind(now)
        .execute(&mut *transaction)
        .await?;
    }
    transaction.commit().await?;
    Ok(())
}
