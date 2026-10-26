//! Shared harness for the integration suites: the real router on the test
//! database with a fixed clock, one mock server on an ephemeral port
//! standing in for ESI, zKillboard and R2Z2, request helpers, the
//! factories and the dataset.

// Not every suite uses every helper.
#![allow(dead_code)]

pub mod dataset;
pub mod factories;
pub mod mock_eve;

use std::path::{Path, PathBuf};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use time::OffsetDateTime;
use time::macros::datetime;
use tower::ServiceExt;

use killfeed::clock::Clock;
use killfeed::config::{BackgroundConfig, Config, DEFAULT_USER_AGENT, EveConfig, RetryTiming};
use killfeed::server::AppState;

pub use mock_eve::MockEve;

/// The site URL of the tests.
pub const APP_URL: &str = "https://killfeed.killmail.space";
pub const HOST: &str = "killfeed.killmail.space";

/// The clock of a test that does not set one.
pub const DEFAULT_NOW: OffsetDateTime = datetime!(2026-10-12 14:30:00 UTC);

pub fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// One app under test: its router, the database it writes to, the clock
/// it reads and the mock EVE services it calls.
pub struct TestApp {
    pub router: Router,
    pub state: AppState,
    pub pool: PgPool,
    pub eve: MockEve,
    pub config: Config,
    pub clock: Clock,
}

pub async fn test_pool() -> PgPool {
    killfeed::db::test_pool()
        .await
        .expect("Postgres not reachable - start it with `docker compose up -d postgres`")
}

/// Clears every table.
pub async fn reset_database(pool: &PgPool) {
    sqlx::query(
        "truncate characters, killmails, killmail_involvements, ship_types, zkillboard_listings,
            listener_state, jobs
         restart identity cascade",
    )
    .execute(pool)
    .await
    .expect("tables truncate");
}

/// The configuration of the tests: every EVE service on the mock, the
/// pauses between retries a millisecond, nothing running in the
/// background (the suites run the jobs and the listener themselves).
pub fn test_config(eve: &MockEve) -> Config {
    Config {
        app_name: "Killfeed".to_owned(),
        max_visible_killmails: 50,
        max_killmail_age_hours: 168,
        discord_invite_url: Some("https://discord.gg/d2NZ5DRDQe".to_owned()),
        eve: EveConfig {
            enabled: true,
            esi_base_url: eve.esi_url(),
            zkillboard_base_url: eve.zkillboard_url(),
            r2z2_base_url: eve.r2z2_url(),
            r2z2_max_consecutive_404s: 10,
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            esi_max_rps: 0,
            timing: RetryTiming {
                esi_killmail_retry: Duration::from_millis(1),
                esi_type_retry: Duration::from_millis(1),
                listener_item_pause: Duration::from_millis(1),
                listener_retry_pause: Duration::from_millis(1),
                listener_throttle_pause: Duration::from_millis(1),
            },
        },
        background: BackgroundConfig {
            listener: false,
            scheduler: false,
            job_workers: 0,
        },
    }
}

impl TestApp {
    /// A fresh app on an empty database, its clock at `DEFAULT_NOW`.
    pub async fn new() -> Self {
        Self::at(DEFAULT_NOW).await
    }

    pub async fn at(now: OffsetDateTime) -> Self {
        let pool = test_pool().await;
        reset_database(&pool).await;
        dataset::create_ts_function(&pool).await;
        let eve = MockEve::start().await;
        let config = test_config(&eve);
        let clock = Clock::fixed(now);
        let state = AppState::new(pool.clone(), config.clone(), clock.clone());
        Self {
            router: killfeed::server::router(state.clone()),
            state,
            pool,
            eve,
            config,
            clock,
        }
    }

    /// Rebuilds the router with a changed configuration, on the same
    /// database, clock and mock.
    pub fn reconfigure(&mut self, change: impl FnOnce(&mut Config)) {
        change(&mut self.config);
        self.state = AppState::new(self.pool.clone(), self.config.clone(), self.clock.clone());
        self.router = killfeed::server::router(self.state.clone());
    }

    pub async fn get(&self, path: &str) -> TestResponse {
        self.get_with(path, &[]).await
    }

    pub async fn get_with(&self, path: &str, headers: &[(&str, &str)]) -> TestResponse {
        let mut all = vec![("host", HOST)];
        all.extend_from_slice(headers);
        self.request(Method::GET, path, header_map(&all), Body::empty())
            .await
    }

    /// A GET as the setup page's fetch sends it.
    pub async fn get_json(&self, path: &str) -> TestResponse {
        self.get_with(path, &[("accept", "application/json")]).await
    }

    pub async fn request(
        &self,
        method: Method,
        path: &str,
        headers: HeaderMap,
        body: Body,
    ) -> TestResponse {
        let mut request = Request::builder().method(method).uri(path);
        for (name, value) in &headers {
            request = request.header(name, value);
        }
        let response = self
            .router
            .clone()
            .oneshot(request.body(body).expect("request builds"))
            .await
            .expect("router answers");
        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body reads")
            .to_bytes()
            .to_vec();
        TestResponse {
            status,
            headers,
            body,
        }
    }

    /// Runs the job queue until nothing is due, the way the queue workers
    /// would.
    pub async fn work_jobs(&self) {
        killfeed::jobs::work_until_idle(&self.state).await;
    }

    /// The queued jobs, oldest first: `(kind, killmail_id)`.
    pub async fn queued_jobs(&self) -> Vec<(String, i64)> {
        sqlx::query_as("select kind, killmail_id from jobs where failed_at is null order by id")
            .fetch_all(&self.pool)
            .await
            .expect("jobs read")
    }

    pub async fn count(&self, table: &str) -> i64 {
        sqlx::query_scalar(sqlx::AssertSqlSafe(format!("select count(*) from {table}")))
            .fetch_one(&self.pool)
            .await
            .expect("count")
    }
}

/// Waits until `check` holds, for at most five seconds (background
/// tasks such as the listener).
pub async fn eventually<F, Fut>(what: &str, mut check: F)
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while !check().await {
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for {what}"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

pub fn header_map(headers: &[(&str, &str)]) -> HeaderMap {
    let mut map = HeaderMap::new();
    for (name, value) in headers {
        map.append(
            axum::http::HeaderName::from_bytes(name.as_bytes()).expect("header name"),
            value.parse().expect("header value"),
        );
    }
    map
}

pub struct TestResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl TestResponse {
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap_or_else(|error| {
            panic!(
                "{} is not JSON ({error}): {}",
                self.status,
                String::from_utf8_lossy(&self.body)
            )
        })
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|value| value.to_str().ok())
    }

    /// The JSON of a 200 answer.
    #[track_caller]
    pub fn ok_json(&self) -> Value {
        assert_eq!(self.status, StatusCode::OK, "{}", self.text());
        self.json()
    }
}

/// Asserts two JSON values are equal with the same key set at every
/// level, naming the first path that differs.
#[track_caller]
pub fn assert_json_eq(expected: &Value, actual: &Value) {
    if let Some(difference) = json_difference("$", expected, actual) {
        panic!("{difference}");
    }
}

pub fn json_difference(path: &str, expected: &Value, actual: &Value) -> Option<String> {
    match (expected, actual) {
        (Value::Object(expected), Value::Object(actual)) => {
            let mut expected_keys: Vec<_> = expected.keys().collect();
            let mut actual_keys: Vec<_> = actual.keys().collect();
            expected_keys.sort();
            actual_keys.sort();
            if expected_keys != actual_keys {
                return Some(format!(
                    "{path}: key sets differ\n  expected {expected_keys:?}\n  actual   {actual_keys:?}"
                ));
            }
            expected.iter().find_map(|(key, value)| {
                json_difference(&format!("{path}.{key}"), value, &actual[key])
            })
        }
        (Value::Array(expected), Value::Array(actual)) => {
            if expected.len() != actual.len() {
                return Some(format!(
                    "{path}: expected {} items, got {}",
                    expected.len(),
                    actual.len()
                ));
            }
            expected
                .iter()
                .zip(actual)
                .enumerate()
                .find_map(|(index, (expected, actual))| {
                    json_difference(&format!("{path}[{index}]"), expected, actual)
                })
        }
        _ if expected == actual => None,
        _ => Some(format!("{path}: expected {expected}, got {actual}")),
    }
}
