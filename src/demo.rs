//! Sample data for the local preview (`seed_demo`, the dev compose's
//! demo-seed service): a character with a week of kills and losses, the
//! characters on them, and the meta groups of the ships involved, so the
//! setup page, the demo and an overlay show something without asking
//! ESI or zKillboard. Safe to repeat: the sample rows are replaced, with
//! times relative to now.

use serde_json::json;
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

/// The character of the sample overlay (`/2112625428`), the id the setup
/// page shows as its placeholder.
pub const DEMO_CHARACTER_ID: i64 = 2112625428;

/// Sample killmail ids, far above real ones.
const FIRST_KILLMAIL_ID: i64 = 9_900_000_000;

/// The anonymous attackers of the bigger fights.
const FIRST_WINGMATE_ID: i64 = 96_000_100;

/// The ships the samples use, with their real meta groups.
const SHIPS: [(i64, i16); 18] = [
    (587, 1),
    (593, 1),
    (602, 1),
    (626, 1),
    (638, 1),
    (645, 1),
    (12044, 2),
    (17738, 4),
    (24698, 1),
    (29984, 14),
    (29990, 14),
    (34562, 14),
    (671, 1),
    (11176, 2),
    (17720, 4),
    (17812, 3),
    (2836, 5),
    (22456, 2),
];

const PILOTS: [(i64, &str); 4] = [
    (DEMO_CHARACTER_ID, "Capsuleer Prime"),
    (95465499, "Attacker One"),
    (93813310, "Victim Pilot"),
    (96000002, "Wingmate Nova"),
];

/// (minutes ago, ship, value, prime is the victim, solo, attackers)
const KILLMAILS: [(i64, i64, f64, bool, bool, usize); 12] = [
    (12, 11176, 48_500_000.0, false, true, 1),
    (47, 17738, 1_250_000_000.0, false, false, 7),
    (95, 587, 2_400_000.0, true, false, 3),
    (180, 12044, 61_000_000.0, false, false, 2),
    (260, 29984, 640_000_000.0, false, false, 12),
    (410, 22456, 72_300_000.0, false, true, 1),
    (620, 17812, 410_000_000.0, true, false, 5),
    (900, 24698, 88_000_000.0, false, false, 4),
    (1500, 34562, 230_000_000.0, false, false, 9),
    (2600, 2836, 3_900_000_000.0, false, false, 21),
    (4400, 626, 14_000_000.0, false, true, 1),
    (7900, 645, 190_000_000.0, true, false, 6),
];

pub async fn seed(pool: &PgPool, now: OffsetDateTime) -> sqlx::Result<usize> {
    let mut transaction = pool.begin().await?;
    sqlx::query("delete from killmails where id >= $1")
        .bind(FIRST_KILLMAIL_ID)
        .execute(&mut *transaction)
        .await?;
    for (type_id, meta_group) in SHIPS {
        sqlx::query(
            "insert into ship_types (type_id, meta_group_id, fetched_at) values ($1, $2, $3)
             on conflict (type_id) do update set meta_group_id = excluded.meta_group_id,
                 fetched_at = excluded.fetched_at",
        )
        .bind(type_id)
        .bind(meta_group)
        .bind(now)
        .execute(&mut *transaction)
        .await?;
    }
    for (id, name) in PILOTS {
        sqlx::query(
            "insert into characters (id, name, created_at, updated_at) values ($1, $2, $3, $3)
             on conflict (id) do update set name = excluded.name",
        )
        .bind(id)
        .bind(name)
        .bind(now)
        .execute(&mut *transaction)
        .await?;
    }
    // The other attackers of the bigger fights, seen on killmails only.
    sqlx::query(
        "insert into characters (id, created_at, updated_at)
         select id, $2, $2 from generate_series($1::bigint, $1 + 24) as id
         on conflict (id) do nothing",
    )
    .bind(FIRST_WINGMATE_ID)
    .bind(now)
    .execute(&mut *transaction)
    .await?;
    for (index, (minutes, ship, value, loss, solo, attackers)) in KILLMAILS.into_iter().enumerate()
    {
        let id = FIRST_KILLMAIL_ID + index as i64;
        let time = now - Duration::minutes(minutes);
        let (victim, attacker) = if loss {
            (DEMO_CHARACTER_ID, 95465499)
        } else {
            (93813310, DEMO_CHARACTER_ID)
        };
        let meta_group = SHIPS
            .iter()
            .find(|(type_id, _)| *type_id == ship)
            .map(|(_, meta_group)| *meta_group);
        let data = json!({
            "killmail_id": id,
            "killmail_time": time
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
            "moon_id": null,
            "solar_system_id": 30002187,
            "victim": {
                "damage_taken": 5000, "items": [], "alliance_id": null, "character_id": victim,
                "corporation_id": 98000001, "faction_id": null, "position": null,
                "ship_type_id": ship
            },
            "attackers": [{
                "alliance_id": null, "character_id": attacker, "corporation_id": 98000002,
                "damage_done": 5000, "faction_id": null, "final_blow": true,
                "security_status": -2.5, "ship_type_id": 587, "weapon_type_id": 3170
            }],
            "war_id": null
        });
        sqlx::query(
            "insert into killmails (id, character_id, hash, data, time, solarsystem_id,
                corporation_id, ship_type_id, ship_meta_group_id, damage_taken, value, solo,
                created_at, updated_at)
             values ($1, $2, $3, $4, $5, 30002187, 98000001, $6, $7, 5000, $8, $9, $5, $5)",
        )
        .bind(id)
        .bind(victim)
        .bind(format!("demo{id}"))
        .bind(&data)
        .bind(time)
        .bind(ship)
        .bind(meta_group)
        .bind(value)
        .bind(solo)
        .execute(&mut *transaction)
        .await?;
        let mut involved = vec![(attacker, true)];
        involved
            .extend((0..attackers.saturating_sub(1) as i64).map(|n| (FIRST_WINGMATE_ID + n, true)));
        involved.push((victim, false));
        for (character_id, is_attacker) in involved {
            sqlx::query(
                "insert into killmail_involvements (killmail_id, character_id, is_attacker, created_at, updated_at)
                 values ($1, $2, $3, $4, $4)",
            )
            .bind(id)
            .bind(character_id)
            .bind(is_attacker)
            .bind(time)
            .execute(&mut *transaction)
            .await?;
        }
    }
    transaction.commit().await?;
    Ok(KILLMAILS.len())
}
