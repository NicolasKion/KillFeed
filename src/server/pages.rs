//! The props of every page, one JSON endpoint per Inertia page of the
//! Laravel app (`/api/pages/<path>`), key for key without the Inertia
//! envelope (`errors` and `auth` were always empty): the setup page, the
//! demo and the overlay. SvelteKit's `+page.server.ts` loads them; the
//! overlay's 10 second poll reloads them, which is what keeps the
//! character watched by the listener.

use axum::Json;
use axum::extract::{Path, RawQuery, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use super::AppState;
use super::errors::{error_page, json_error, server_error, wants_json};
use crate::characters::{self, ResolveError};
use crate::jobs::enqueue_backfill_meta_group;
use crate::json::php_numbers;
use crate::killmails::feed;
use crate::options::OverlayOptions;
use crate::ship_types::demo_ships;
use crate::zkillboard::queue_recent_killmails;

fn props(value: Value) -> Response {
    Json(php_numbers(value)).into_response()
}

fn options(state: &AppState, query: Option<&str>, default_summary: bool) -> OverlayOptions {
    OverlayOptions::from_query(
        query,
        state.config.max_visible_killmails,
        state.config.max_killmail_age_hours,
        default_summary,
    )
}

/// Laravel's `abort(404)`: `{"message": ""}` for JSON clients and the
/// api, the 404 page otherwise.
pub fn abort_not_found(headers: &HeaderMap, api: bool) -> Response {
    if api || wants_json(headers) {
        json_error(StatusCode::NOT_FOUND, "")
    } else {
        error_page(StatusCode::NOT_FOUND)
    }
}

/// The setup page (`Home`).
pub async fn home(State(state): State<AppState>) -> Response {
    props(json!({
        "max_visible_killmails": state.config.max_visible_killmails,
        "max_killmail_age_hours": state.config.max_killmail_age_hours,
        "discord_invite_url": state.config.discord_invite_url,
        "sim_ships": demo_ships(&state).await,
    }))
}

/// The simulated overlay (`Demo`); the summary panel is on by default.
pub async fn demo(State(state): State<AppState>, RawQuery(query): RawQuery) -> Response {
    let options = options(&state, query.as_deref(), true);
    props(json!({
        "count": options.count,
        "hours": options.hours,
        "summary": options.summary,
        "meta": options.meta,
        "sim_ships": demo_ships(&state).await,
    }))
}

/// The overlay (`Character`), in Laravel's order: the character (from
/// ESI when it has no name yet), the zKillboard backfill, the character
/// marked as listening, the totals and the latest killmails.
pub async fn character(
    State(state): State<AppState>,
    Path(character): Path<String>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
) -> Response {
    let options = options(&state, query.as_deref(), false);
    let Some(id) = characters::parse_id(&character) else {
        return abort_not_found(&headers, true);
    };
    let mut character = match characters::resolve(&state, id).await {
        Ok(character) => character,
        Err(ResolveError::NotFound) => return abort_not_found(&headers, true),
        Err(ResolveError::Database(error)) => return server_error(error),
    };
    queue_recent_killmails(&state, character.id, options.count).await;
    if let Err(error) = characters::listen_now(&state, &mut character).await {
        return server_error(error);
    }
    let stats = match feed::stats(&state, character.id, &options).await {
        Ok(stats) => stats,
        Err(error) => return server_error(error),
    };
    let (killmails, missing_meta_group) = match feed::latest(&state, character.id, &options).await {
        Ok(latest) => latest,
        Err(error) => return server_error(error),
    };
    // Killmails stored before meta groups were tracked heal in the
    // background.
    for killmail_id in missing_meta_group {
        if let Err(error) = enqueue_backfill_meta_group(&state, killmail_id).await {
            tracing::error!(killmail_id, %error, "meta group heal not queued");
        }
    }
    props(json!({
        "count": options.count,
        "hours": options.hours,
        "summary": options.summary,
        "meta": options.meta,
        "killmails_count": stats["killmails_count"],
        "kills_value": stats["kills_value"],
        "losses_value": stats["losses_value"],
        "character": character.to_json(),
        "killmails": killmails,
    }))
}

/// The preview the setup page fetches while an id is typed: id, name and
/// portrait URL; a character ESI does not know is a 404.
pub async fn preview(
    State(state): State<AppState>,
    Path(character): Path<String>,
    uri: axum::http::Uri,
    headers: HeaderMap,
) -> Response {
    if character.is_empty() || !character.bytes().all(|byte| byte.is_ascii_digit()) {
        // `whereNumber`: the route does not match.
        return super::errors::not_found(uri, headers).await;
    }
    let Some(id) = characters::parse_id(&character) else {
        return abort_not_found(&headers, false);
    };
    match characters::resolve(&state, id).await {
        Ok(character) => Json(characters::preview(&character)).into_response(),
        Err(ResolveError::NotFound) => abort_not_found(&headers, false),
        Err(ResolveError::Database(error)) => server_error(error),
    }
}
