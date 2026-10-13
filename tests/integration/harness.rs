//! The harness itself: every route of the Laravel app is known to the
//! api (a route not ported yet answers 501, never 404), and the ones
//! dropped with Horizon and Sanctum answer 404.

use axum::body::Body;
use axum::http::{Method, StatusCode};

use crate::common::golden::read_json;
use crate::common::{HOST, TestApp, header_map};

/// The api path a Laravel route is served at: the pages' props come from
/// `/api/pages/<path>`, the rest at their own path.
fn api_path(uri: &str) -> Option<String> {
    let path = match uri {
        "/" => "/api/pages/home".to_owned(),
        "demo" => "/api/pages/demo".to_owned(),
        "{character}" => "/api/pages/2112625428".to_owned(),
        "characters/{character}/preview" => "/characters/2112625428/preview".to_owned(),
        "up" => "/up".to_owned(),
        _ => return None,
    };
    Some(path)
}

fn dropped(uri: &str) -> bool {
    uri.starts_with("horizon") || uri.starts_with("sanctum")
}

#[tokio::test]
async fn every_laravel_route_is_routed() {
    let app = TestApp::new().await;
    let routes = read_json("routes.json");
    for route in routes.as_array().expect("routes") {
        let uri = route["uri"].as_str().expect("a uri");
        let method = route["method"].as_str().expect("a method");
        let method = method.split('|').next().expect("a method");
        if dropped(uri) {
            continue;
        }
        let path = api_path(uri).unwrap_or_else(|| panic!("{uri} has no api path"));
        let response = app
            .request(
                method.parse::<Method>().expect("a method"),
                &path,
                header_map(&[("host", HOST)]),
                Body::empty(),
            )
            .await;
        assert_ne!(response.status, StatusCode::NOT_FOUND, "{method} {path}");
        assert_ne!(
            response.status,
            StatusCode::METHOD_NOT_ALLOWED,
            "{method} {path}"
        );
    }
}

/// FIXES.md (deliberate changes): Horizon never ran and Sanctum was never
/// used; their routes are gone.
#[tokio::test]
async fn the_horizon_and_sanctum_routes_are_gone() {
    let app = TestApp::new().await;
    let routes = read_json("routes.json");
    for route in routes.as_array().expect("routes") {
        let uri = route["uri"].as_str().expect("a uri");
        if !dropped(uri) {
            continue;
        }
        let path = format!(
            "/{}",
            uri.replace("{view?}", "")
                .replace("{id}", "1")
                .replace("{tag}", "x")
        );
        let method = route["method"].as_str().expect("a method");
        let method = method.split('|').next().expect("a method");
        let response = app
            .request(
                method.parse::<Method>().expect("a method"),
                path.trim_end_matches('/'),
                header_map(&[("host", HOST)]),
                Body::empty(),
            )
            .await;
        assert_eq!(response.status, StatusCode::NOT_FOUND, "{method} {path}");
    }
}
