//! The HTTP side: shared state, the router and the cross-cutting layers.
//! Pages are rendered by SvelteKit; the api serves their props as JSON
//! (`/api/pages/*`), the character preview the setup page asks for, and
//! the health checks.

pub mod errors;
pub mod health;
pub mod pages;

use std::sync::Arc;

use axum::Router;
use axum::extract::Request;
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use sqlx::PgPool;
use tokio::sync::Notify;
use tower_http::normalize_path::NormalizePath;

use crate::clock::Clock;
use crate::config::Config;
use crate::eve::Eve;

/// The default bind address of the API server; the SvelteKit dev proxy and
/// the reverse proxy point at it.
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3000";

/// The listen address from `BIND_ADDR`, with the local default.
pub fn bind_addr() -> String {
    std::env::var("BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_owned())
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub clock: Clock,
    /// The clients of ESI, zKillboard and R2Z2.
    pub eve: Eve,
    /// Wakes the queue workers when a job is queued.
    pub jobs: Arc<Notify>,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config, clock: Clock) -> Self {
        Self {
            pool,
            eve: Eve::new(&config.eve, clock.clone()),
            config: Arc::new(config),
            clock,
            jobs: Arc::new(Notify::new()),
        }
    }
}

/// The app with every route. A trailing slash is ignored before routing
/// (`/up/` is `/up`), as Laravel's router ignored it.
pub fn router(state: AppState) -> Router {
    let routes = Router::new()
        .route("/up", get(health::up))
        .route("/characters/{character}/preview", get(pages::preview))
        .nest("/api", api_router())
        .fallback(errors::not_found)
        .method_not_allowed_fallback(errors::method_not_allowed)
        .with_state(state.clone());
    Router::new()
        .fallback_service(NormalizePath::trim_trailing_slash(routes))
        .layer(axum::middleware::from_fn_with_state(
            state,
            reload_inertia_requests,
        ))
}

/// One props endpoint per page, at the page's own path under
/// `/api/pages` (the setup page is `/api/pages/home`).
fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/pages/home", get(pages::home))
        .route("/pages/demo", get(pages::demo))
        .route("/pages/{character}", get(pages::character))
}

/// OBS overlays (and browsers) left open on the Laravel site poll with
/// Inertia's XHR (`X-Inertia: true`); Caddy sends every such request
/// here. A 409 with `X-Inertia-Location` (the full URL, query and
/// trailing slash kept) makes Inertia load the page in full, now from
/// SvelteKit, before anything else happens (FIXES.md).
async fn reload_inertia_requests(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if !request.headers().contains_key("x-inertia") {
        return next.run(request).await;
    }
    let headers = request.headers();
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_else(|| state.config.app_url.split("://").next().unwrap_or("https"));
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_else(|| state.config.app_url.split("://").nth(1).unwrap_or_default());
    let path = request
        .uri()
        .path_and_query()
        .map_or("/", |path| path.as_str());
    let location = format!("{scheme}://{host}{path}");
    let mut response = StatusCode::CONFLICT.into_response();
    if let Ok(location) = location.parse() {
        response
            .headers_mut()
            .insert("x-inertia-location", location);
    }
    response
}
