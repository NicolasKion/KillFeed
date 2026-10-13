//! Characters: a known one by its stored name, an unknown one by ESI's
//! public character info. Any ESI failure means not found, a connection
//! error included.

use serde_json::{Value, json};
use time::OffsetDateTime;

use crate::json::optional_timestamp;
use crate::server::AppState;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Character {
    pub id: i64,
    pub name: Option<String>,
    pub listening_since: Option<OffsetDateTime>,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

impl Character {
    /// The character as the props carry it.
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "listening_since": optional_timestamp(self.listening_since),
            "created_at": optional_timestamp(self.created_at),
            "updated_at": optional_timestamp(self.updated_at),
        })
    }
}

#[derive(Debug)]
pub enum ResolveError {
    NotFound,
    Database(sqlx::Error),
}

impl From<sqlx::Error> for ResolveError {
    fn from(error: sqlx::Error) -> Self {
        ResolveError::Database(error)
    }
}

/// A character id from the URL: any digit string (leading zeros too, as
/// `whereNumber` matched them); one beyond 64 bits is no EVE id.
pub fn parse_id(text: &str) -> Option<i64> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// A character with a name, from the table or from ESI (then stored with
/// it).
pub async fn resolve(state: &AppState, id: i64) -> Result<Character, ResolveError> {
    let known: Option<Character> = sqlx::query_as(
        "select id, name, listening_since, created_at, updated_at
         from characters where id = $1 and name is not null",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;
    if let Some(character) = known {
        return Ok(character);
    }

    let answer = match state.eve.esi_get(&format!("/characters/{id}")).await {
        Ok(answer) if answer.ok() => answer,
        Ok(answer) => {
            tracing::debug!(id, status = answer.status, "character not found on ESI");
            return Err(ResolveError::NotFound);
        }
        Err(error) => {
            tracing::warn!(id, %error, "character lookup failed");
            return Err(ResolveError::NotFound);
        }
    };
    let name = answer
        .json()
        .ok()
        .and_then(|body| body["name"].as_str().map(str::to_owned));

    // `updateOrCreate`: a new row gets both timestamps, an existing one a
    // new `updated_at` only when the name changed.
    let now = state.clock.now();
    let character = sqlx::query_as(
        "insert into characters (id, name, created_at, updated_at) values ($1, $2, $3, $3)
         on conflict (id) do update set
             name = excluded.name,
             updated_at = case when characters.name is distinct from excluded.name
                               then excluded.updated_at else characters.updated_at end
         returning id, name, listening_since, created_at, updated_at",
    )
    .bind(id)
    .bind(name)
    .bind(now)
    .fetch_one(&state.pool)
    .await?;
    Ok(character)
}

/// `listenNow()`: the overlay is open, the listener watches the
/// character for the next five minutes.
pub async fn listen_now(state: &AppState, character: &mut Character) -> sqlx::Result<()> {
    let now = state.clock.now();
    if character.listening_since == Some(now) {
        return Ok(());
    }
    sqlx::query("update characters set listening_since = $2, updated_at = $2 where id = $1")
        .bind(character.id)
        .bind(now)
        .execute(&state.pool)
        .await?;
    character.listening_since = Some(now);
    character.updated_at = Some(now);
    Ok(())
}

/// The character preview of the setup page.
pub fn preview(character: &Character) -> Value {
    json!({
        "id": character.id,
        "name": character.name,
        "portrait": format!("https://images.evetech.net/characters/{}/portrait?size=256", character.id),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_id;

    #[test]
    fn reads_ids_like_where_number() {
        assert_eq!(parse_id("2112625428"), Some(2112625428));
        assert_eq!(parse_id("02112625428"), Some(2112625428));
        assert_eq!(parse_id("99999999999999999999"), None);
        assert_eq!(parse_id("12a"), None);
        assert_eq!(parse_id("-1"), None);
        assert_eq!(parse_id(""), None);
    }
}
