//! The HTTP side: shared state, the router and the cross-cutting layers.
//! Pages are rendered by SvelteKit; the api serves their props as JSON
//! (`/api/pages/*`), the character preview the setup page asks for, and
//! the health checks.

pub mod errors;
pub mod health;

use std::sync::Arc;

use axum::Router;
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
        .route(
            "/characters/{character}/preview",
            get(errors::not_implemented),
        )
        .nest("/api", api_router())
        .fallback(errors::not_found)
        .method_not_allowed_fallback(errors::method_not_allowed)
        .with_state(state);
    Router::new().fallback_service(NormalizePath::trim_trailing_slash(routes))
}

/// One props endpoint per page, at the page's own path under
/// `/api/pages` (the setup page is `/api/pages/home`).
fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/pages/home", get(errors::not_implemented))
        .route("/pages/demo", get(errors::not_implemented))
        .route("/pages/{character}", get(errors::not_implemented))
}
