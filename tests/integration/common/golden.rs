//! The golden fixtures recorded from the Laravel reference
//! (scripts/laravel-reference/record.py), the dataset they start from and
//! the normalizations every comparison applies. The only accepted
//! differences are listed here and in FIXES.md.
//!
//! - N1: Inertia's envelope is not part of the props: `component`, `url`,
//!   `version`, `sharedProps` and the shared `errors` and `auth` props
//!   (always `{}` and `{user: null}`, nothing reads them).
//! - N2: timestamps the request wrote (`listening_since`, `updated_at`,
//!   `created_at` later than the recording moment) are `NOW`: the
//!   reference wrote its own clock seconds after the recording started,
//!   the Rust suites write their fixed clock, the recording moment.
//! - N3: the reference's own origin (`http://127.0.0.1:8316`) is APP_URL
//!   (record.py rewrote it).
//! - N4: ESI is called at its unversioned paths (`/characters/1` with
//!   `X-Compatibility-Date`) instead of `/latest/characters/1/` (FIXES.md,
//!   deliberate changes); the outgoing requests compare after mapping.
//! - N5: the health page's render time.
//! - N6: the involvements of a killmail in id order. Laravel loaded them
//!   without an order, and MySQL answered in whichever index order its
//!   plan picked (by character id on some pages, by id on others); the
//!   api orders them by id.
//! - N7: rows read back with SQL (not through the api) write their
//!   numbers as PHP's json_encode did: a float without a fraction is an
//!   integer.

use serde_json::{Map, Value};
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Duration, OffsetDateTime};

use super::{TestApp, repo_path};

pub fn read_json(relative: &str) -> Value {
    let path = repo_path(&format!("tests/fixtures/{relative}"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

pub fn read_text(relative: &str) -> String {
    let path = repo_path(&format!("tests/fixtures/{relative}"));
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The moment the fixtures were recorded.
pub fn recorded_now() -> OffsetDateTime {
    let meta = read_json("golden/meta.json");
    OffsetDateTime::parse(meta["now"].as_str().expect("now"), &Rfc3339).expect("a moment")
}

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
    /// A fresh app with tests/fixtures/dataset.json seeded, the mock
    /// answering what the reference's fakes answered, and the clock at the
    /// moment the golden fixtures were recorded.
    pub async fn with_dataset() -> Self {
        let now = recorded_now();
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

    /// Every character as Eloquent serialized the model, by id.
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

    /// Every killmail with its involvements, as Eloquent serialized
    /// `Killmail::with('killmailInvolvements')`, by id (N7).
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
        php_numbers(&rows.unwrap_or(Value::Null))
    }
}

/// Creates the `ts()` SQL helper the row readers use: Eloquent's date
/// serialization (`2026-10-12T14:00:00.000000Z`).
pub async fn create_ts_function(pool: &sqlx::PgPool) {
    sqlx::query(
        "create or replace function ts(timestamptz) returns text language sql immutable
         as $$ select to_char($1 at time zone 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') $$",
    )
    .execute(pool)
    .await
    .expect("ts() creates");
}

/// N7: integral floats as integers, as PHP's json_encode writes them.
pub fn php_numbers(value: &Value) -> Value {
    match value {
        Value::Number(number) => match number.as_f64() {
            Some(float)
                if number.is_f64()
                    && float.fract() == 0.0
                    && float.abs() < 9.007_199_254_740_992e15 =>
            {
                Value::from(float as i64)
            }
            _ => value.clone(),
        },
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, value)| (key.clone(), php_numbers(value)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(php_numbers).collect()),
        other => other.clone(),
    }
}

/// N6: every `killmail_involvements` list in id order.
pub fn involvements_by_id(value: &Value) -> Value {
    match value {
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, value)| {
                    let mut value = involvements_by_id(value);
                    if key == "killmail_involvements"
                        && let Value::Array(items) = &mut value
                    {
                        items.sort_by_key(|item| item["id"].as_i64());
                    }
                    (key.clone(), value)
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(involvements_by_id).collect()),
        other => other.clone(),
    }
}

/// N2: every Eloquent timestamp later than the recording moment is `NOW`.
pub fn written_now(value: &Value, recorded: OffsetDateTime) -> Value {
    match value {
        Value::String(text) => match parse_eloquent(text) {
            Some(moment) if moment >= recorded => Value::String("NOW".to_owned()),
            _ => value.clone(),
        },
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, value)| (key.clone(), written_now(value, recorded)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| written_now(item, recorded))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn parse_eloquent(text: &str) -> Option<OffsetDateTime> {
    if text.len() != 27 || !text.ends_with('Z') || text.as_bytes()[10] != b'T' {
        return None;
    }
    OffsetDateTime::parse(text, &Rfc3339).ok()
}

/// One recorded page.
pub struct GoldenPage {
    pub path: String,
    pub status: u16,
    /// The props after N1.
    pub props: Option<Value>,
    pub characters: Value,
    pub jobs: Value,
    pub requests: Vec<String>,
}

pub fn page(name: &str) -> GoldenPage {
    let recorded = read_json(&format!("golden/pages/{name}.json"));
    let props = recorded["page"]["props"].as_object().map(|props| {
        let mut props: Map<String, Value> = props.clone();
        props.remove("errors");
        props.remove("auth");
        Value::Object(props)
    });
    GoldenPage {
        path: recorded["path"].as_str().expect("a path").to_owned(),
        status: recorded["status"].as_u64().expect("a status") as u16,
        props,
        characters: recorded["characters"].clone(),
        jobs: recorded["jobs"].clone(),
        requests: requests(&recorded["requests"]),
    }
}

/// N4: the URLs the reference called, as the paths the mock sees.
pub fn requests(recorded: &Value) -> Vec<String> {
    recorded
        .as_array()
        .map(|urls| {
            urls.iter()
                .map(|url| {
                    let url = url.as_str().expect("a URL");
                    if let Some(path) = url.strip_prefix("https://esi.evetech.net/latest/") {
                        format!("/esi/{}", path.trim_end_matches('/'))
                    } else if let Some(path) = url.strip_prefix("https://zkillboard.com/") {
                        format!("/zkillboard/{path}")
                    } else {
                        panic!("an unexpected URL: {url}")
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The job kinds of the rewrite for Laravel's job classes.
pub fn job_kind(class: &str) -> &'static str {
    match class {
        "ProcessKillmail" => "process_killmail",
        "BackfillShipMetaGroup" => "backfill_meta_group",
        other => panic!("an unknown job: {other}"),
    }
}

/// The props endpoint of a page path.
pub fn api_path(path: &str) -> String {
    match path.split_once('?') {
        Some(("/", query)) => format!("/api/pages/home?{query}"),
        None if path == "/" => "/api/pages/home".to_owned(),
        _ => format!("/api/pages{path}"),
    }
}
