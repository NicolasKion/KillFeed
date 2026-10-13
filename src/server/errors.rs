//! Error answers: JSON under `/api` and for clients that ask for JSON
//! (the setup page's preview fetch sends `Accept: application/json`),
//! Laravel's minimal error page everywhere else.

use axum::Json;
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{Html, IntoResponse, Response};
use serde_json::json;

/// Laravel's minimal error view with the status and its message.
const ERROR_PAGE: &str = include_str!("error.html");

/// The message of a status, as Laravel's error pages show it.
pub fn message(status: StatusCode) -> &'static str {
    match status.as_u16() {
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        419 => "Page Expired",
        429 => "Too Many Requests",
        503 => "Service Unavailable",
        _ => "Server Error",
    }
}

pub fn json_error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "message": message }))).into_response()
}

pub fn error_page(status: StatusCode) -> Response {
    let page = ERROR_PAGE
        .replace("{{status}}", status.as_str())
        .replace("{{message}}", message(status));
    (status, Html(page)).into_response()
}

/// Whether the client asked for JSON (Laravel's `expectsJson`).
pub fn wants_json(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|accept| accept.contains("/json") || accept.contains("+json"))
}

/// Any failure the client cannot fix: logged, answered with the generic
/// message.
pub fn server_error(error: impl std::fmt::Display) -> Response {
    tracing::error!(%error, "request failed");
    json_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        message(StatusCode::INTERNAL_SERVER_ERROR),
    )
}

/// Unknown paths: JSON for the api and for JSON clients (with the route
/// in the message, as Laravel words it), the 404 page otherwise.
pub async fn not_found(uri: Uri, headers: HeaderMap) -> Response {
    if uri.path() == "/api" || uri.path().starts_with("/api/") {
        json_error(StatusCode::NOT_FOUND, message(StatusCode::NOT_FOUND))
    } else if wants_json(&headers) {
        let route = uri.path().trim_start_matches('/');
        json_error(
            StatusCode::NOT_FOUND,
            &format!("The route {route} could not be found."),
        )
    } else {
        error_page(StatusCode::NOT_FOUND)
    }
}

/// A known path with another method.
pub async fn method_not_allowed() -> Response {
    error_page(StatusCode::METHOD_NOT_ALLOWED)
}

/// The answer of every route that is registered but not ported yet.
pub async fn not_implemented() -> Response {
    json_error(StatusCode::NOT_IMPLEMENTED, "Not Implemented")
}
