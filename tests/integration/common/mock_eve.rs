//! A local stand-in for the EVE services on one ephemeral port: ESI under
//! `/esi`, zKillboard under `/zkillboard` and the R2Z2 firehose under
//! `/r2z2`. Tests tell it what to answer per path (a lasting answer, or
//! answers used once in order), can make every request fail, and read the
//! requests it got. A path nobody set up answers 404, like ESI does for an
//! unknown id.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Reply {
    pub status: u16,
    pub body: Option<Value>,
    pub headers: Vec<(String, String)>,
}

impl Reply {
    pub fn json(body: Value) -> Self {
        Self {
            status: 200,
            body: Some(body),
            headers: Vec::new(),
        }
    }

    pub fn status(status: u16) -> Self {
        Self {
            status,
            body: None,
            headers: Vec::new(),
        }
    }

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }
}

/// One request the mock received.
#[derive(Clone, Debug)]
pub struct Received {
    pub method: String,
    /// The path with the service prefix (`/esi/characters/1`).
    pub path: String,
    pub query: Option<String>,
    pub headers: HeaderMap,
}

impl Received {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|value| value.to_str().ok())
    }
}

#[derive(Default)]
struct Inner {
    lasting: HashMap<String, Reply>,
    once: HashMap<String, VecDeque<Reply>>,
    fail_with: Option<u16>,
    received: Vec<Received>,
}

#[derive(Clone)]
pub struct MockEve {
    port: u16,
    inner: Arc<Mutex<Inner>>,
}

impl MockEve {
    pub async fn start() -> Self {
        let inner = Arc::new(Mutex::new(Inner::default()));
        let app = Router::new().fallback(answer).with_state(inner.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("mock binds");
        let port = listener.local_addr().expect("mock address").port();
        tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });
        Self { port, inner }
    }

    fn base(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub fn esi_url(&self) -> String {
        format!("{}/esi", self.base())
    }

    pub fn zkillboard_url(&self) -> String {
        format!("{}/zkillboard", self.base())
    }

    pub fn r2z2_url(&self) -> String {
        format!("{}/r2z2", self.base())
    }

    /// Answers `path` (with its service prefix) with `reply` from now on.
    pub fn on(&self, path: &str, reply: Reply) {
        self.inner
            .lock()
            .expect("mock state")
            .lasting
            .insert(path.to_owned(), reply);
    }

    /// Answers the next request to `path` with `reply`, before any lasting
    /// answer; several are used in order.
    pub fn once(&self, path: &str, reply: Reply) {
        self.inner
            .lock()
            .expect("mock state")
            .once
            .entry(path.to_owned())
            .or_default()
            .push_back(reply);
    }

    /// Every request answers `status` until `recover`.
    pub fn fail_with(&self, status: u16) {
        self.inner.lock().expect("mock state").fail_with = Some(status);
    }

    pub fn recover(&self) {
        self.inner.lock().expect("mock state").fail_with = None;
    }

    pub fn received(&self) -> Vec<Received> {
        self.inner.lock().expect("mock state").received.clone()
    }

    /// The paths requested, in order.
    pub fn paths(&self) -> Vec<String> {
        self.received().into_iter().map(|r| r.path).collect()
    }

    /// The paths requested under a service prefix (`/esi`).
    pub fn paths_under(&self, prefix: &str) -> Vec<String> {
        self.paths()
            .into_iter()
            .filter(|path| path.starts_with(&format!("{prefix}/")))
            .collect()
    }

    pub fn forget_requests(&self) {
        self.inner.lock().expect("mock state").received.clear();
    }

    /// The shared answers of tests/fixtures/eve.json: `esi:<path>` and
    /// `zkillboard:<path>`, with dataset times resolved against `now`.
    pub fn load_fixture(&self, now: time::OffsetDateTime) {
        let fakes = super::dataset::read_json("eve.json");
        for (key, answer) in fakes.as_object().expect("an object") {
            let (service, path) = key.split_once(':').expect("service:path");
            let path = match service {
                "esi" => format!("/esi/{path}"),
                _ => format!("/zkillboard/{path}"),
            };
            let reply = Reply {
                status: answer["status"].as_u64().unwrap_or(200) as u16,
                body: Some(super::dataset::resolve(&answer["body"], now)),
                headers: Vec::new(),
            };
            self.on(&path, reply);
        }
    }

    /// The ESI meta group answers of a type: the meta group attribute
    /// (1692) and the tech level (422) when given.
    pub fn esi_type(&self, type_id: i64, meta_group: Option<i64>, tech_level: Option<i64>) {
        let mut attributes = Vec::new();
        if let Some(meta_group) = meta_group {
            attributes
                .push(serde_json::json!({ "attribute_id": 1692, "value": meta_group as f64 }));
        }
        if let Some(tech_level) = tech_level {
            attributes.push(serde_json::json!({ "attribute_id": 422, "value": tech_level as f64 }));
        }
        self.on(
            &format!("/esi/universe/types/{type_id}"),
            Reply::json(serde_json::json!({ "type_id": type_id, "dogma_attributes": attributes })),
        );
    }
}

async fn answer(
    State(inner): State<Arc<Mutex<Inner>>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let mut inner = inner.lock().expect("mock state");
    let path = uri.path().to_owned();
    inner.received.push(Received {
        method: method.to_string(),
        path: path.clone(),
        query: uri.query().map(str::to_owned),
        headers,
    });
    if let Some(status) = inner.fail_with {
        return StatusCode::from_u16(status)
            .expect("a status")
            .into_response();
    }
    let reply = inner
        .once
        .get_mut(&path)
        .and_then(VecDeque::pop_front)
        .or_else(|| inner.lasting.get(&path).cloned())
        .unwrap_or_else(|| {
            Reply::json(serde_json::json!({ "error": "not found" })).with_status(404)
        });
    let mut response = Response::builder().status(reply.status);
    for (name, value) in &reply.headers {
        response = response.header(name, value);
    }
    match reply.body {
        Some(body) => response
            .header("content-type", "application/json")
            .body(Body::from(body.to_string())),
        None => response.body(Body::empty()),
    }
    .expect("a response")
}

impl Reply {
    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }
}
