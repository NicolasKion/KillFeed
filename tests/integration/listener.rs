//! The R2Z2 firehose listener (`app:listen-for-kills`), which the PHP
//! suite never covered, against the mock firehose: where it starts, how
//! it waits for and skips missing sequences, retries failures, which
//! killmails it queues and how it stops.

use serde_json::{Value, json};
use time::Duration;
use tokio_util::sync::CancellationToken;

use killfeed::listener;

use crate::common::factories::CharacterFactory;
use crate::common::mock_eve::Reply;
use crate::common::{TestApp, eventually};

/// A firehose item for killmail `id` with a victim and one attacker.
fn item(id: i64, victim: i64, attacker: i64) -> Value {
    json!({
        "killmail_id": id,
        "hash": format!("hash{id}"),
        "esi": {
            "killmail_id": id,
            "killmail_time": "2026-10-12T14:00:00Z",
            "solar_system_id": 30002187,
            "victim": { "character_id": victim, "corporation_id": 98000001,
                        "damage_taken": 500, "ship_type_id": 670, "items": [] },
            "attackers": [{ "character_id": attacker, "corporation_id": 98000002,
                            "damage_done": 500, "final_blow": true, "security_status": -2.5,
                            "ship_type_id": 587, "weapon_type_id": 3170 }]
        },
        "zkb": {
            "hash": format!("hash{id}"), "locationID": 40009081, "fittedValue": 1.0,
            "totalValue": 2.0, "droppedValue": 0.0, "destroyedValue": 2.0, "points": 1,
            "npc": false, "solo": true, "awox": false, "labels": []
        },
        "uploaded_at": 1773950479,
        "sequence_id": id
    })
}

async fn stored_sequence(app: &TestApp) -> Option<i64> {
    sqlx::query_scalar("select sequence from listener_state where name = 'r2z2'")
        .fetch_optional(&app.pool)
        .await
        .expect("reads")
}

async fn store_sequence(app: &TestApp, sequence: i64) {
    sqlx::query(
        "insert into listener_state (name, sequence, updated_at) values ('r2z2', $1, now())",
    )
    .bind(sequence)
    .execute(&app.pool)
    .await
    .expect("inserts");
}

struct Running {
    cancel: CancellationToken,
    task: tokio::task::JoinHandle<()>,
}

fn start(app: &TestApp) -> Running {
    let cancel = CancellationToken::new();
    let task = tokio::spawn(listener::run(app.state.clone(), cancel.clone()));
    Running { cancel, task }
}

impl Running {
    async fn stop(self) {
        self.cancel.cancel();
        tokio::time::timeout(std::time::Duration::from_secs(5), self.task)
            .await
            .expect("the listener stops")
            .expect("the listener does not panic");
    }
}

#[tokio::test]
async fn starts_at_the_current_sequence_without_a_stored_one() {
    let app = TestApp::new().await;
    app.eve.on(
        "/r2z2/sequence.json",
        Reply::json(json!({ "sequence": 100 })),
    );
    app.eve.on("/r2z2/100.json", Reply::json(item(100, 1, 2)));

    let running = start(&app);
    eventually("sequence 100 handled", || async {
        stored_sequence(&app).await == Some(100)
    })
    .await;
    running.stop().await;

    assert_eq!(
        app.eve.paths_under("/r2z2")[..3],
        ["/r2z2/sequence.json", "/r2z2/100.json", "/r2z2/101.json"]
    );
}

#[tokio::test]
async fn resumes_after_the_stored_sequence() {
    let app = TestApp::new().await;
    store_sequence(&app, 41).await;
    app.eve.on("/r2z2/42.json", Reply::json(item(42, 1, 2)));

    let running = start(&app);
    eventually("sequence 42 handled", || async {
        stored_sequence(&app).await == Some(42)
    })
    .await;
    running.stop().await;

    assert_eq!(app.eve.paths_under("/r2z2")[0], "/r2z2/42.json");
    assert!(!app.eve.paths().contains(&"/r2z2/sequence.json".to_owned()));
}

/// A killmail is queued, with the zKillboard and ESI data of the item,
/// when its victim or an attacker has had an overlay open in the last
/// five minutes; the rest are only read.
#[tokio::test]
async fn queues_only_the_killmails_of_watched_characters() {
    let app = TestApp::new().await;
    let now = app.clock.now();
    let victim = CharacterFactory::listening(now - Duration::minutes(4))
        .create(&app.pool, now)
        .await;
    let attacker = CharacterFactory::listening(now - Duration::minutes(1))
        .create(&app.pool, now)
        .await;
    let stale = CharacterFactory::listening(now - Duration::minutes(6))
        .create(&app.pool, now)
        .await;
    store_sequence(&app, 9).await;
    app.eve
        .on("/r2z2/10.json", Reply::json(item(500, victim, 1)));
    app.eve
        .on("/r2z2/11.json", Reply::json(item(501, 2, attacker)));
    app.eve
        .on("/r2z2/12.json", Reply::json(item(502, stale, 3)));
    app.eve.on("/r2z2/13.json", Reply::json(item(503, 4, 5)));

    let running = start(&app);
    eventually("sequence 13 handled", || async {
        stored_sequence(&app).await == Some(13)
    })
    .await;
    running.stop().await;

    assert_eq!(
        app.queued_jobs().await,
        vec![
            ("process_killmail".to_owned(), 500),
            ("process_killmail".to_owned(), 501)
        ]
    );
    let payload: Value = sqlx::query_scalar("select payload from jobs where killmail_id = 500")
        .fetch_one(&app.pool)
        .await
        .expect("reads");
    assert_eq!(payload["hash"], "hash500");
    assert_eq!(payload["zkb"]["total_value"], 2.0);
    assert_eq!(payload["esi"]["victim"]["character_id"], victim);
    assert_eq!(app.count("killmails").await, 0, "the workers store them");
}

/// Ten 404s in a row (6 seconds apart), then the listener asks where the
/// firehose is and goes on from there.
#[tokio::test]
async fn missing_sequences_are_waited_for_then_skipped() {
    let app = TestApp::new().await;
    store_sequence(&app, 199).await;
    app.eve.once(
        "/r2z2/sequence.json",
        Reply::json(json!({ "sequence": 300 })),
    );
    app.eve.on("/r2z2/300.json", Reply::json(item(300, 1, 2)));

    let running = start(&app);
    eventually("sequence 300 handled", || async {
        stored_sequence(&app).await == Some(300)
    })
    .await;
    running.stop().await;

    let paths = app.eve.paths_under("/r2z2");
    let mut expected = vec!["/r2z2/200.json".to_owned(); 10];
    expected.push("/r2z2/sequence.json".to_owned());
    expected.push("/r2z2/300.json".to_owned());
    assert_eq!(paths[..12], expected[..]);
}

/// A failure other than 404 is retried on the same sequence.
#[tokio::test]
async fn a_failure_is_retried_on_the_same_sequence() {
    let app = TestApp::new().await;
    store_sequence(&app, 6).await;
    app.eve.once("/r2z2/7.json", Reply::status(502));
    app.eve.once("/r2z2/7.json", Reply::status(500));
    app.eve.on("/r2z2/7.json", Reply::json(item(7, 1, 2)));

    let running = start(&app);
    eventually("sequence 7 handled", || async {
        stored_sequence(&app).await == Some(7)
    })
    .await;
    running.stop().await;

    assert_eq!(app.eve.paths_under("/r2z2")[..3], ["/r2z2/7.json"; 3]);
}

/// FIXES.md: an unreachable firehose is retried instead of ending the
/// listener.
#[tokio::test]
async fn a_connection_error_is_retried() {
    let mut app = TestApp::new().await;
    app.reconfigure(|config| config.eve.r2z2_base_url = "http://127.0.0.1:1".to_owned());
    store_sequence(&app, 1).await;

    let running = start(&app);
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(!running.task.is_finished(), "the listener keeps trying");
    running.stop().await;

    assert_eq!(stored_sequence(&app).await, Some(1));
}

/// FIXES.md: an item that does not parse is logged and skipped.
#[tokio::test]
async fn a_broken_item_is_skipped() {
    let app = TestApp::new().await;
    store_sequence(&app, 19).await;
    app.eve
        .on("/r2z2/20.json", Reply::json(json!({ "unexpected": true })));
    app.eve.on("/r2z2/21.json", Reply::json(item(21, 1, 2)));

    let running = start(&app);
    eventually("sequence 21 handled", || async {
        stored_sequence(&app).await == Some(21)
    })
    .await;
    running.stop().await;

    assert_eq!(
        app.eve.paths_under("/r2z2")[..2],
        ["/r2z2/20.json", "/r2z2/21.json"]
    );
}

/// Shutdown (SIGTERM in the api) ends the loop after the current item,
/// with the sequence stored, replacing `app:stop-listening`.
#[tokio::test]
async fn stops_when_asked() {
    let app = TestApp::new().await;
    store_sequence(&app, 1).await;
    app.eve.on("/r2z2/2.json", Reply::json(item(2, 1, 2)));

    let running = start(&app);
    eventually("sequence 2 handled", || async {
        stored_sequence(&app).await == Some(2)
    })
    .await;
    running.stop().await;
    let requests = app.eve.received().len();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert_eq!(app.eve.received().len(), requests, "nothing after the stop");
    assert_eq!(stored_sequence(&app).await, Some(2));
}
