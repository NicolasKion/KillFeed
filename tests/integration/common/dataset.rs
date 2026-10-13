//! tests/fixtures/dataset.json, five characters and six killmails with
//! the edge cases between them, seeded into the test database. Times are
//! minutes from `DATASET_NOW`, the clock of a test on the dataset.

use serde_json::Value;
use time::macros::datetime;
use time::macros::format_description;
use time::{Duration, OffsetDateTime};

use super::{TestApp, repo_path};

pub fn read_json(relative: &str) -> Value {
    let path = repo_path(&format!("tests/fixtures/{relative}"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The clock of a test on the dataset.
pub const DATASET_NOW: OffsetDateTime = datetime!(2026-10-13 12:15:33 UTC);

/// A dataset value: `{offset_minutes}` is a moment relative to `now`, as
/// a Postgres timestamp, or with `esi` as ESI writes `killmail_time`.
pub fn resolve(value: &Value, now: OffsetDateTime) -> Value {
    match value {
        Value::Object(fields) if fields.contains_key("offset_minutes") => {
            let moment =
                now + Duration::minutes(fields["offset_minutes"].as_i64().expect("minutes"));
            let text = if fields.get("esi").and_then(Value::as_bool) == Some(true) {
                moment
                    .format(format_description!(
                        "[year]-[month]-[day]T[hour]:[minute]:[second]Z"
                    ))
                    .expect("formats")
            } else {
                moment
                    .format(format_description!(
                        "[year]-[month]-[day] [hour]:[minute]:[second]+00"
                    ))
                    .expect("formats")
            };
            Value::String(text)
        }
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, value)| (key.clone(), resolve(value, now)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(|item| resolve(item, now)).collect()),
        other => other.clone(),
    }
}

impl TestApp {
    /// A fresh app with the dataset seeded, the mock answering
    /// tests/fixtures/eve.json and the clock at `DATASET_NOW`.
    pub async fn with_dataset() -> Self {
        let now = DATASET_NOW;
        let app = Self::at(now).await;
        app.seed_dataset().await;
        app.eve.load_fixture(now);
        app
    }

    pub async fn seed_dataset(&self) {
        let dataset = read_json("dataset.json");
        let now = self.clock.now();
        for table in ["characters", "killmails", "killmail_involvements"] {
            for row in dataset[table].as_array().expect("rows") {
                sqlx::query(sqlx::AssertSqlSafe(format!(
                    "insert into {table} select * from jsonb_populate_record(null::{table}, $1)"
                )))
                .bind(resolve(row, now))
                .execute(&self.pool)
                .await
                .unwrap_or_else(|error| panic!("{table}: {error}"));
            }
        }
        sqlx::query(
            "select setval(pg_get_serial_sequence('killmail_involvements', 'id'),
                (select max(id) from killmail_involvements))",
        )
        .execute(&self.pool)
        .await
        .expect("sequence resets");
    }

    /// Every character as the api serializes it, by id.
    pub async fn character_rows(&self) -> Value {
        let rows: Option<Value> = sqlx::query_scalar(
            "select coalesce(json_agg(json_build_object(
                'id', id, 'name', name, 'listening_since', ts(listening_since),
                'created_at', ts(created_at), 'updated_at', ts(updated_at)) order by id), '[]')
             from characters",
        )
        .fetch_one(&self.pool)
        .await
        .expect("characters read");
        rows.unwrap_or(Value::Null)
    }

    /// Every killmail with its involvements, as the api serializes them,
    /// by id.
    pub async fn killmail_rows(&self) -> Value {
        let rows: Option<Value> = sqlx::query_scalar(
            "select coalesce(json_agg(json_build_object(
                'id', k.id, 'character_id', k.character_id, 'hash', k.hash, 'data', k.data,
                'time', ts(k.time), 'solarsystem_id', k.solarsystem_id,
                'corporation_id', k.corporation_id, 'alliance_id', k.alliance_id,
                'ship_type_id', k.ship_type_id, 'ship_meta_group_id', k.ship_meta_group_id,
                'damage_taken', k.damage_taken, 'value', k.value, 'solo', k.solo::int,
                'created_at', ts(k.created_at), 'updated_at', ts(k.updated_at),
                'killmail_involvements', (
                    select coalesce(json_agg(json_build_object(
                        'id', i.id, 'killmail_id', i.killmail_id, 'character_id', i.character_id,
                        'is_attacker', i.is_attacker, 'created_at', ts(i.created_at),
                        'updated_at', ts(i.updated_at)) order by i.id), '[]')
                    from killmail_involvements i where i.killmail_id = k.id)
             ) order by k.id), '[]')
             from killmails k",
        )
        .fetch_one(&self.pool)
        .await
        .expect("killmails read");
        killfeed::json::whole_numbers(rows.unwrap_or(Value::Null))
    }
}

/// Creates the `ts()` SQL helper the row readers use: the api's date
/// format (`2026-10-12T14:00:00.000000Z`).
pub async fn create_ts_function(pool: &sqlx::PgPool) {
    sqlx::query(
        "create or replace function ts(timestamptz) returns text language sql immutable
         as $$ select to_char($1 at time zone 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') $$",
    )
    .execute(pool)
    .await
    .expect("ts() creates");
}
