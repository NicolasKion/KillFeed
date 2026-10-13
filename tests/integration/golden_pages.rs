//! Every page's props against the golden fixtures recorded from Laravel
//! (`tests/fixtures/golden/pages`), one test per fixture, on the dataset
//! with the clock at the recording and the mock answering what the
//! reference's fakes answered. The props come from `/api/pages/<path>`;
//! the rows the request left (characters, queued jobs) and the requests
//! it sent compare too. Differences: N1, N2 and N4 (`common::golden`)
//! and the two 500s Laravel answered (FIXES.md).

use serde_json::{Value, json};

use crate::common::golden::{self, GoldenPage, api_path, job_kind, written_now};
use crate::common::{TestApp, assert_json_eq};

/// The jobs queued, as the recorder wrote them.
async fn queued(app: &TestApp) -> Value {
    let rows: Vec<(String, i64, bool, bool)> = sqlx::query_as(
        "select kind, killmail_id, payload->'zkb' is not null and payload->>'zkb' <> 'null',
                payload->'esi' is not null and payload->>'esi' <> 'null'
         from jobs order by id",
    )
    .fetch_all(&app.pool)
    .await
    .expect("jobs read");
    Value::Array(
        rows.into_iter()
            .map(|(kind, killmail_id, with_zkill, with_esi)| {
                json!({ "job": kind, "killmail_id": killmail_id,
                        "with_zkill": with_zkill, "with_esi": with_esi })
            })
            .collect(),
    )
}

fn expected_jobs(recorded: &Value) -> Value {
    Value::Array(
        recorded
            .as_array()
            .expect("jobs")
            .iter()
            .map(|job| {
                let mut job = job.clone();
                job["job"] = json!(job_kind(job["job"].as_str().expect("a class")));
                job
            })
            .collect(),
    )
}

async fn check(name: &str) {
    let golden: GoldenPage = golden::page(name);
    let now = golden::recorded_now();
    let app = TestApp::with_dataset().await;

    let response = app.get(&api_path(&golden.path)).await;

    assert_eq!(
        response.status.as_u16(),
        golden.status,
        "{}",
        response.text()
    );
    if let Some(expected) = &golden.props {
        assert_json_eq(
            &written_now(expected, now),
            &written_now(&response.json(), now),
        );
    }
    assert_json_eq(
        &written_now(&golden.characters, now),
        &written_now(&app.character_rows().await, now),
    );
    assert_json_eq(&expected_jobs(&golden.jobs), &queued(&app).await);
    assert_eq!(golden.requests, app.eve.paths(), "the outgoing requests");
}

macro_rules! golden_pages {
    ($($test:ident => $name:literal,)*) => {
        $(
            #[tokio::test]
            async fn $test() {
                check($name).await;
            }
        )*

        const NAMES: &[&str] = &[$($name,)* "character-overflow", "character-zkillboard-down"];
    };
}

golden_pages! {
    home => "home",
    demo => "demo",
    demo_clamped => "demo-clamped",
    demo_trailing_slash => "demo-trailing-slash",
    demo_query_00 => "demo-query-00",
    demo_query_01 => "demo-query-01",
    demo_query_02 => "demo-query-02",
    demo_query_03 => "demo-query-03",
    demo_query_04 => "demo-query-04",
    demo_query_05 => "demo-query-05",
    demo_query_06 => "demo-query-06",
    demo_query_07 => "demo-query-07",
    demo_query_08 => "demo-query-08",
    demo_query_09 => "demo-query-09",
    demo_query_10 => "demo-query-10",
    demo_query_11 => "demo-query-11",
    demo_query_12 => "demo-query-12",
    demo_query_13 => "demo-query-13",
    demo_query_14 => "demo-query-14",
    demo_query_15 => "demo-query-15",
    demo_query_16 => "demo-query-16",
    demo_query_17 => "demo-query-17",
    character => "character",
    character_count_2 => "character-count-2",
    character_hours_1 => "character-hours-1",
    character_options => "character-options",
    character_leading_zeros => "character-leading-zeros",
    character_trailing_slash => "character-trailing-slash",
    character_resolved => "character-resolved",
    character_attacker => "character-attacker",
    character_unknown => "character-unknown",
}

/// FIXES.md: Laravel answered 500 when zKillboard failed (five tries);
/// the overlay now renders from the stored killmails after the same five
/// tries, and the character is marked as listening.
#[tokio::test]
async fn character_zkillboard_down() {
    let golden = golden::page("character-zkillboard-down");
    assert_eq!(golden.status, 500);
    let now = golden::recorded_now();
    let app = TestApp::with_dataset().await;

    let response = app.get(&api_path(&golden.path)).await;

    assert_json_eq(
        &json!({
            "count": 50, "hours": 168, "summary": false, "meta": true,
            "killmails_count": 0, "kills_value": 0, "losses_value": 0,
            "character": {
                "id": 90000001, "name": "Quiet Pilot", "listening_since": "NOW",
                "created_at": "2026-10-11T10:15:33.000000Z", "updated_at": "NOW"
            },
            "killmails": []
        })
        .as_object()
        .map(|props| {
            let mut props = props.clone();
            props["character"]["created_at"] = golden.characters[0]["created_at"].clone();
            Value::Object(props)
        })
        .expect("props"),
        &written_now(&response.ok_json(), now),
    );
    assert_eq!(golden.requests, app.eve.paths(), "five tries, as before");
    assert_eq!(app.count("jobs").await, 0);
}

/// FIXES.md: an id beyond 64 bits answered 500; it is a 404 now, without
/// asking anyone.
#[tokio::test]
async fn character_overflow() {
    let golden = golden::page("character-overflow");
    assert_eq!(golden.status, 500);
    let app = TestApp::with_dataset().await;

    let response = app.get(&api_path(&golden.path)).await;

    assert_eq!(response.status, 404);
    assert!(app.eve.received().is_empty());
}

/// Every recorded page has its test.
#[test]
fn every_recorded_page_is_compared() {
    let directory = crate::common::repo_path("tests/fixtures/golden/pages");
    for entry in std::fs::read_dir(directory).expect("the fixtures") {
        let name = entry.expect("an entry").file_name();
        let name = name.to_string_lossy();
        let name = name.trim_end_matches(".json");
        assert!(NAMES.contains(&name), "{name} has no test");
    }
}
