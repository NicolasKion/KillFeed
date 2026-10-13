//! A type's meta group from ESI's dogma attributes, for the overlay's
//! corner badges (`ShipTypeService`): the meta group attribute (1692) for
//! faction, officer and storyline hulls, else the tech level (422), since
//! ESI leaves 1692 out on Tech II and III hulls. Kept 30 days per type in
//! `ship_types` (static data only changes with expansions); a failed
//! lookup is Tech I and is not kept, so the next one asks again.

use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;
use time::Duration;

use crate::eve::EveError;
use crate::server::AppState;

/// The meta group attribute (faction, officer, storyline).
pub const META_GROUP_ATTRIBUTE_ID: i64 = 1692;
/// The tech level attribute; ESI omits 1692 on Tech II/III hulls.
pub const TECH_LEVEL_ATTRIBUTE_ID: i64 = 422;

pub const TECH_I: i16 = 1;
pub const TECH_II: i16 = 2;
pub const TECH_III: i16 = 14;

/// How long a looked up meta group is kept.
const KEEP_FOR: Duration = Duration::days(30);

/// Tries of a lookup (Laravel's `Http::retry(3, 500)`).
const TRIES: usize = 3;

/// The ships of the simulated feed (`demo_ship_type_ids`): frigate to
/// titan, with recognizable renders on images.evetech.net.
pub const DEMO_SHIP_TYPE_IDS: [i64; 13] = [
    587, 593, 602, 626, 638, 645, 12044, 17738, 24698, 29984, 29990, 34562, 671,
];

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SimShip {
    pub type_id: i64,
    pub meta_group_id: i16,
}

/// The meta group of a type.
pub async fn meta_group_id(state: &AppState, type_id: i64) -> i16 {
    lookup_meta_group(state, type_id, true)
        .await
        .unwrap_or(TECH_I)
}

/// The kept meta group of a type, else ESI's when `ask_esi`; `None` when
/// there is neither.
async fn lookup_meta_group(state: &AppState, type_id: i64, ask_esi: bool) -> Option<i16> {
    let now = state.clock.now();
    let kept: Result<Option<i16>, sqlx::Error> = sqlx::query_scalar(
        "select meta_group_id from ship_types where type_id = $1 and fetched_at > $2",
    )
    .bind(type_id)
    .bind(now - KEEP_FOR)
    .fetch_optional(&state.pool)
    .await;
    match kept {
        Ok(Some(meta_group)) => return Some(meta_group),
        Ok(None) => {}
        Err(error) => tracing::error!(%error, type_id, "ship type not read"),
    }
    if !ask_esi {
        return None;
    }

    let attributes = lookup(state, type_id).await?;
    let meta_group = from_attributes(&attributes);
    let stored = sqlx::query(
        "insert into ship_types (type_id, meta_group_id, fetched_at) values ($1, $2, $3)
         on conflict (type_id) do update
         set meta_group_id = excluded.meta_group_id, fetched_at = excluded.fetched_at",
    )
    .bind(type_id)
    .bind(meta_group)
    .bind(now)
    .execute(&state.pool)
    .await;
    if let Err(error) = stored {
        tracing::error!(%error, type_id, "ship type not stored");
    }
    Some(meta_group)
}

/// The dogma attributes of a type from ESI, `None` when ESI failed three
/// times.
async fn lookup(state: &AppState, type_id: i64) -> Option<HashMap<i64, f64>> {
    for attempt in 1..=TRIES {
        match state
            .eve
            .esi_get(&format!("/universe/types/{type_id}"))
            .await
        {
            Ok(answer) if answer.ok() => {
                // A body without attributes (or without JSON) is a type
                // without any: Tech I, as Laravel's `?? []` read it.
                let body = answer.json().unwrap_or(Value::Null);
                return Some(
                    body["dogma_attributes"]
                        .as_array()
                        .map(|attributes| {
                            attributes
                                .iter()
                                .filter_map(|attribute| {
                                    Some((
                                        attribute["attribute_id"].as_i64()?,
                                        attribute["value"].as_f64()?,
                                    ))
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                );
            }
            Ok(answer) => {
                tracing::debug!(
                    type_id,
                    status = answer.status,
                    attempt,
                    "type lookup failed"
                )
            }
            // Switched off or holding back for the error limit: trying
            // again changes nothing.
            Err(error @ (EveError::Disabled | EveError::ErrorLimit)) => {
                tracing::debug!(type_id, %error, "type lookup skipped");
                return None;
            }
            Err(error) => tracing::debug!(type_id, %error, attempt, "type lookup failed"),
        }
        if attempt < TRIES {
            tokio::time::sleep(state.config.eve.timing.esi_type_retry).await;
        }
    }
    tracing::warn!(type_id, "type lookup failed, falling back to Tech I");
    None
}

fn from_attributes(attributes: &HashMap<i64, f64>) -> i16 {
    if let Some(meta_group) = attributes.get(&META_GROUP_ATTRIBUTE_ID) {
        return *meta_group as i16;
    }
    match attributes
        .get(&TECH_LEVEL_ATTRIBUTE_ID)
        .map_or(1, |level| *level as i64)
    {
        2 => TECH_II,
        3 => TECH_III,
        _ => TECH_I,
    }
}

/// The demo feed's ships with their meta groups. Once a lookup fails
/// (ESI is down), the rest of this view does not ask ESI again: one
/// failed lookup per setup page view instead of thirteen.
pub async fn demo_ships(state: &AppState) -> Vec<SimShip> {
    let mut ships = Vec::with_capacity(DEMO_SHIP_TYPE_IDS.len());
    let mut ask_esi = true;
    for type_id in DEMO_SHIP_TYPE_IDS {
        let meta_group = lookup_meta_group(state, type_id, ask_esi).await;
        ask_esi &= meta_group.is_some();
        ships.push(SimShip {
            type_id,
            meta_group_id: meta_group.unwrap_or(TECH_I),
        });
    }
    ships
}
