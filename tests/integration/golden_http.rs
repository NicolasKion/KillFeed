//! The answers that are not Inertia pages against the golden fixtures
//! (`tests/fixtures/golden/http`): the health page, the preview JSON, the
//! api's error pages, the dropped package routes and the 409 of a stale
//! Inertia client.

use axum::http::StatusCode;

use crate::common::golden::{self, read_json, read_text, requests, written_now};
use crate::common::{TestApp, assert_json_eq};

/// N5: the render time of the health page.
fn render_time(html: &str) -> String {
    let marker = "Response rendered in ";
    let Some(start) = html.find(marker) else {
        return html.to_owned();
    };
    let rest = &html[start + marker.len()..];
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    format!("{}{marker}Nms{}", &html[..start], &rest[digits..])
}

/// One recorded answer, replayed against the api: same status, content
/// type, body, rows and outgoing requests.
async fn check(name: &str) {
    let recorded = read_json(&format!("golden/http/{name}.json"));
    let now = golden::recorded_now();
    let app = TestApp::with_dataset().await;
    let headers: Vec<(String, String)> = recorded["request_headers"]
        .as_object()
        .expect("headers")
        .iter()
        .map(|(name, value)| {
            (
                name.to_lowercase(),
                value.as_str().expect("a value").to_owned(),
            )
        })
        .collect();
    let headers: Vec<(&str, &str)> = headers
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();

    let response = app
        .get_with(recorded["path"].as_str().expect("a path"), &headers)
        .await;

    assert_eq!(
        response.status.as_u16() as u64,
        recorded["status"].as_u64().expect("a status"),
        "{}",
        response.text()
    );
    let content_type = recorded["content_type"].as_str().expect("a content type");
    assert_eq!(
        response
            .header("content-type")
            .map(|value| value.replace(' ', "")),
        Some(content_type.replace(' ', "")),
    );
    if let Some(json) = recorded.get("json") {
        assert_json_eq(json, &response.json());
    } else {
        let expected = read_text(&format!("golden/http/{name}.html"));
        assert_eq!(render_time(&expected), render_time(&response.text()));
    }
    assert_json_eq(
        &written_now(&recorded["characters"], now),
        &written_now(&app.character_rows().await, now),
    );
    assert_eq!(requests(&recorded["requests"]), app.eve.paths());
}

macro_rules! golden_http {
    ($($test:ident => $name:literal,)*) => {
        $(
            #[tokio::test]
            async fn $test() {
                check($name).await;
            }
        )*

        const NAMES: &[&str] = &[
            $($name,)* "horizon", "horizon-api", "sanctum-csrf-cookie", "inertia-stale"
        ];
    };
}

golden_http! {
    up => "up",
    not_found => "not-found",
    not_found_json => "not-found-json",
    preview_known => "preview-known",
    preview_resolved => "preview-resolved",
    preview_trailing_slash => "preview-trailing-slash",
    preview_unknown => "preview-unknown",
    preview_unknown_html => "preview-unknown-html",
    preview_not_numeric => "preview-not-numeric",
    preview_not_numeric_html => "preview-not-numeric-html",
}

/// FIXES.md (deliberate changes): Horizon answered 403 and Sanctum's
/// cookie route 204; both are gone.
#[tokio::test]
async fn the_dropped_package_routes_are_not_found() {
    let app = TestApp::new().await;
    for name in ["horizon", "horizon-api", "sanctum-csrf-cookie"] {
        let recorded = read_json(&format!("golden/http/{name}.json"));
        assert_ne!(recorded["status"], 404);
        let path = recorded["path"].as_str().expect("a path");
        assert_eq!(app.get(path).await.status, StatusCode::NOT_FOUND, "{path}");
    }
}

/// FIXES.md (deliberate changes): a stale Inertia client gets the 409
/// with the full URL Laravel answered, before any side effect (Laravel ran
/// the page first, asking zKillboard).
#[tokio::test]
async fn inertia_stale() {
    let recorded = read_json("golden/http/inertia-stale.json");
    let app = TestApp::with_dataset().await;
    let path = recorded["path"].as_str().expect("a path");

    let response = app
        .get_with(
            path,
            &[("x-inertia", "true"), ("x-inertia-version", "stale")],
        )
        .await;

    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(
        response.header("x-inertia-location"),
        recorded["x_inertia_location"].as_str()
    );
    assert!(app.eve.received().is_empty());
    let untouched: bool =
        sqlx::query_scalar("select listening_since is null from characters where id = 2112625428")
            .fetch_one(&app.pool)
            .await
            .expect("reads");
    assert!(untouched, "the page did not run");
}

/// The api paths answer the same 409 (Caddy sends every request with the
/// header to the api), the page's props path included.
#[tokio::test]
async fn an_inertia_request_reloads_any_path() {
    let app = TestApp::new().await;
    for path in ["/", "/demo?summary=1", "/123/?count=5", "/api/pages/demo"] {
        let response = app.get_with(path, &[("x-inertia", "true")]).await;
        assert_eq!(response.status, StatusCode::CONFLICT, "{path}");
        assert_eq!(
            response.header("x-inertia-location"),
            Some(format!("https://killfeed.killmail.space{path}").as_str())
        );
    }
}

/// FIXES.md (deliberate changes): no session or XSRF cookies.
#[tokio::test]
async fn pages_set_no_cookies() {
    let app = TestApp::with_dataset().await;
    for path in [
        "/api/pages/home",
        "/api/pages/demo",
        "/api/pages/2112625428",
        "/characters/2112625428/preview",
        "/up",
    ] {
        let response = app.get(path).await;
        assert_eq!(response.status, StatusCode::OK, "{path}");
        assert!(response.header("set-cookie").is_none(), "{path}");
    }
}

/// Every recorded answer has its test.
#[test]
fn every_recorded_answer_is_compared() {
    let directory = crate::common::repo_path("tests/fixtures/golden/http");
    for entry in std::fs::read_dir(directory).expect("the fixtures") {
        let name = entry.expect("an entry").file_name();
        let name = name.to_string_lossy();
        if let Some(name) = name.strip_suffix(".json") {
            assert!(NAMES.contains(&name), "{name} has no test");
        }
    }
}
