//! The answers that are not page props: the health page, the preview
//! JSON on the dataset, the error pages and the cookies.

use axum::http::StatusCode;
use serde_json::json;

use crate::common::{TestApp, TestResponse, assert_json_eq};

#[track_caller]
fn assert_content_type(response: &TestResponse, expected: &str) {
    assert_eq!(response.header("content-type"), Some(expected));
}

#[track_caller]
fn assert_not_found_page(response: &TestResponse) {
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_content_type(response, "text/html; charset=utf-8");
    let page = response.text();
    assert!(page.contains("<title>Not Found</title>"), "{page}");
    assert!(page.contains("404                    </h1>"), "{page}");
    assert!(
        page.contains("Not Found                    </div>"),
        "{page}"
    );
}

#[tokio::test]
async fn up() {
    let app = TestApp::with_dataset().await;

    let response = app.get("/up").await;

    assert_eq!(response.status, StatusCode::OK);
    assert_content_type(&response, "text/html; charset=utf-8");
    let page = response.text();
    assert!(page.contains("<title>Killfeed</title>"), "{page}");
    assert!(page.contains("Application up"), "{page}");
    assert!(page.contains("Response rendered in "), "{page}");
    assert!(app.eve.paths().is_empty());
}

#[tokio::test]
async fn an_unknown_path_is_the_not_found_page() {
    let app = TestApp::with_dataset().await;
    for path in ["/nope", "/characters/abc/preview"] {
        assert_not_found_page(&app.get(path).await);
    }
    assert!(app.eve.paths().is_empty());
}

#[tokio::test]
async fn an_unknown_path_names_the_route_for_json_clients() {
    let app = TestApp::with_dataset().await;

    let response = app.get_json("/nope").await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_content_type(&response, "application/json");
    assert_json_eq(
        &json!({ "message": "The route nope could not be found." }),
        &response.json(),
    );

    let response = app.get_json("/characters/abc/preview").await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_json_eq(
        &json!({ "message": "The route characters/abc/preview could not be found." }),
        &response.json(),
    );
    assert!(app.eve.paths().is_empty());
}

/// A stored character is answered from the database, with or without a
/// trailing slash.
#[tokio::test]
async fn preview_known() {
    let app = TestApp::with_dataset().await;
    for path in [
        "/characters/2112625428/preview",
        "/characters/2112625428/preview/",
    ] {
        let response = app.get_json(path).await;
        assert_content_type(&response, "application/json");
        assert_json_eq(
            &json!({
                "id": 2112625428_i64,
                "name": "Capsuleer Prime",
                "portrait": "https://images.evetech.net/characters/2112625428/portrait?size=256"
            }),
            &response.ok_json(),
        );
    }
    assert!(app.eve.paths().is_empty());
}

/// A stored character without a name gets it from ESI, and keeps it.
#[tokio::test]
async fn preview_resolved() {
    let app = TestApp::with_dataset().await;

    let response = app.get_json("/characters/93813310/preview").await;

    assert_json_eq(
        &json!({
            "id": 93813310,
            "name": "Victim Pilot",
            "portrait": "https://images.evetech.net/characters/93813310/portrait?size=256"
        }),
        &response.ok_json(),
    );
    let name: Option<String> =
        sqlx::query_scalar("select name from characters where id = 93813310")
            .fetch_one(&app.pool)
            .await
            .expect("reads");
    assert_eq!(name.as_deref(), Some("Victim Pilot"));
    assert_eq!(app.eve.paths(), ["/esi/characters/93813310"]);
}

/// An id ESI does not know: `{"message": ""}` for the setup page's fetch,
/// the 404 page in a browser.
#[tokio::test]
async fn preview_unknown() {
    let app = TestApp::with_dataset().await;

    let response = app.get_json("/characters/1/preview").await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_content_type(&response, "application/json");
    assert_json_eq(&json!({ "message": "" }), &response.json());

    assert_not_found_page(&app.get("/characters/1/preview").await);
    assert_eq!(app.eve.paths(), ["/esi/characters/1", "/esi/characters/1"]);
    assert_eq!(app.count("characters").await, 5);
}

/// No session or XSRF cookies anywhere.
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
