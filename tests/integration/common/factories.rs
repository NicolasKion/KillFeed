//! The model factories the PHP tests used (database/factories), as plain
//! inserts with the same defaults: a character with a name and no
//! listening time, a killmail of the last hour with an empty `data`, an
//! involvement as the victim.

use std::sync::atomic::{AtomicI64, Ordering};

use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

static NEXT_CHARACTER: AtomicI64 = AtomicI64::new(90_000_000);
static NEXT_KILLMAIL: AtomicI64 = AtomicI64::new(100_000_000);

pub struct CharacterFactory {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub listening_since: Option<OffsetDateTime>,
}

impl Default for CharacterFactory {
    fn default() -> Self {
        Self {
            id: None,
            name: Some("Test Pilot".to_owned()),
            listening_since: None,
        }
    }
}

impl CharacterFactory {
    pub fn named(name: &str) -> Self {
        Self {
            name: Some(name.to_owned()),
            ..Self::default()
        }
    }

    /// `Character::factory()->listening()`.
    pub fn listening(now: OffsetDateTime) -> Self {
        Self {
            listening_since: Some(now),
            ..Self::default()
        }
    }

    pub async fn create(self, pool: &PgPool, now: OffsetDateTime) -> i64 {
        let id = self
            .id
            .unwrap_or_else(|| NEXT_CHARACTER.fetch_add(1, Ordering::Relaxed));
        sqlx::query(
            "insert into characters (id, name, listening_since, created_at, updated_at)
             values ($1, $2, $3, $4, $4)",
        )
        .bind(id)
        .bind(self.name)
        .bind(self.listening_since)
        .bind(now)
        .execute(pool)
        .await
        .expect("character inserts");
        id
    }
}

pub struct KillmailFactory {
    pub id: Option<i64>,
    pub character_id: Option<i64>,
    pub minutes_ago: i64,
    pub ship_type_id: i64,
    pub ship_meta_group_id: Option<i16>,
    pub value: Option<f64>,
    pub solo: bool,
    pub data: serde_json::Value,
}

impl Default for KillmailFactory {
    fn default() -> Self {
        Self {
            id: None,
            character_id: None,
            minutes_ago: 30,
            ship_type_id: 587,
            ship_meta_group_id: Some(1),
            value: Some(1_000_000.0),
            solo: false,
            data: serde_json::json!([]),
        }
    }
}

impl KillmailFactory {
    pub async fn create(self, pool: &PgPool, now: OffsetDateTime) -> i64 {
        let id = self
            .id
            .unwrap_or_else(|| NEXT_KILLMAIL.fetch_add(1, Ordering::Relaxed));
        sqlx::query(
            "insert into killmails (id, character_id, hash, data, time, solarsystem_id,
                corporation_id, alliance_id, ship_type_id, ship_meta_group_id, damage_taken,
                value, solo, created_at, updated_at)
             values ($1, $2, $3, $4, $5, 30002187, 98000001, null, $6, $7, 5000, $8, $9, $10, $10)",
        )
        .bind(id)
        .bind(self.character_id)
        .bind(format!("hash{id}"))
        .bind(self.data)
        .bind(now - Duration::minutes(self.minutes_ago))
        .bind(self.ship_type_id)
        .bind(self.ship_meta_group_id)
        .bind(self.value)
        .bind(self.solo)
        .bind(now)
        .execute(pool)
        .await
        .expect("killmail inserts");
        id
    }
}

/// `$killmail->killmailInvolvements()->create([...])`.
pub async fn involve(
    pool: &PgPool,
    killmail_id: i64,
    character_id: i64,
    is_attacker: bool,
    now: OffsetDateTime,
) {
    sqlx::query(
        "insert into killmail_involvements (killmail_id, character_id, is_attacker, created_at, updated_at)
         values ($1, $2, $3, $4, $4)",
    )
    .bind(killmail_id)
    .bind(character_id)
    .bind(is_attacker)
    .bind(now)
    .execute(pool)
    .await
    .expect("involvement inserts");
}
