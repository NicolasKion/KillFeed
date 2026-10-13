//! The EVE services the app calls: ESI (character names, type dogma,
//! killmails), zKillboard (a character's latest killmails) and its R2Z2
//! firehose. One HTTP client with the app's User-Agent for all three.
//!
//! ESI is called at its unversioned routes with a pinned compatibility
//! date. The client keeps to `ESI_MAX_RPS` and, when ESI reports fewer
//! than 10 errors left in its error window, stops calling it until the
//! window resets: past 100 errors a minute ESI answers 420 to every route
//! for the whole server, and the setup page's preview turns every unknown
//! id typed into an error.
//!
//! With `EVE_APIS_ENABLED` off every call fails at once without touching
//! the network.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use time::OffsetDateTime;

use crate::clock::Clock;
use crate::config::EveConfig;

/// The ESI behavior the app is written against (`esi_spec.py dates`).
pub const ESI_COMPATIBILITY_DATE: &str = "2026-08-18";

/// Errors left in ESI's window below which no call goes out.
const ESI_ERROR_LIMIT_FLOOR: i64 = 10;

/// How long an outgoing call may take.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// The answer of an outgoing call: its status and body.
#[derive(Debug, Clone)]
pub struct Answer {
    pub status: u16,
    pub body: Vec<u8>,
}

impl Answer {
    /// A 2xx.
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    pub fn json(&self) -> Result<serde_json::Value, EveError> {
        serde_json::from_slice(&self.body).map_err(|error| EveError::Parse(error.to_string()))
    }
}

/// Why a call has no answer.
#[derive(Debug, Clone, PartialEq)]
pub enum EveError {
    /// `EVE_APIS_ENABLED` is off.
    Disabled,
    /// No connection, a timeout, a broken answer.
    Transport(String),
    /// ESI's error window is nearly used up.
    ErrorLimit,
    /// The body is not the JSON expected.
    Parse(String),
}

impl std::fmt::Display for EveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EveError::Disabled => write!(f, "the EVE services are switched off"),
            EveError::Transport(error) => write!(f, "transport: {error}"),
            EveError::ErrorLimit => write!(f, "ESI's error limit is nearly used up"),
            EveError::Parse(error) => write!(f, "unexpected answer: {error}"),
        }
    }
}

impl std::error::Error for EveError {}

#[derive(Default)]
struct EsiGate {
    /// No ESI call before this moment (the error window's reset).
    closed_until: Option<OffsetDateTime>,
    /// The earliest moment of the next call, for `ESI_MAX_RPS`.
    next_slot: Option<tokio::time::Instant>,
}

#[derive(Clone)]
pub struct Eve {
    client: reqwest::Client,
    config: Arc<EveConfig>,
    clock: Clock,
    gate: Arc<Mutex<EsiGate>>,
}

impl Eve {
    pub fn new(config: &EveConfig, clock: Clock) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(config.user_agent.clone())
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .expect("an HTTP client");
        Self {
            client,
            config: Arc::new(config.clone()),
            clock,
            gate: Arc::new(Mutex::new(EsiGate::default())),
        }
    }

    /// `GET` an ESI route (`/characters/2112625428`).
    pub async fn esi_get(&self, path: &str) -> Result<Answer, EveError> {
        if !self.config.enabled {
            return Err(EveError::Disabled);
        }
        self.wait_for_esi()?.await;
        let url = format!("{}{path}", self.config.esi_base_url);
        let response = self
            .client
            .get(url)
            .header("X-Compatibility-Date", ESI_COMPATIBILITY_DATE)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|error| EveError::Transport(error.to_string()))?;
        self.read_error_limit(response.headers());
        read(response).await
    }

    /// `GET` a zKillboard path (`/api/characterID/1/`).
    pub async fn zkillboard_get(&self, path: &str) -> Result<Answer, EveError> {
        self.get(&format!("{}{path}", self.config.zkillboard_base_url))
            .await
    }

    /// `GET` a firehose file (`/sequence.json`, `/100.json`).
    pub async fn r2z2_get(&self, path: &str) -> Result<Answer, EveError> {
        self.get(&format!("{}{path}", self.config.r2z2_base_url))
            .await
    }

    async fn get(&self, url: &str) -> Result<Answer, EveError> {
        if !self.config.enabled {
            return Err(EveError::Disabled);
        }
        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|error| EveError::Transport(error.to_string()))?;
        read(response).await
    }

    /// Refuses while the error window is nearly used up; otherwise the
    /// wait for the next slot of `ESI_MAX_RPS`.
    fn wait_for_esi(&self) -> Result<tokio::time::Sleep, EveError> {
        let mut gate = self.gate.lock().expect("ESI gate");
        if let Some(until) = gate.closed_until {
            if self.clock.now() < until {
                return Err(EveError::ErrorLimit);
            }
            gate.closed_until = None;
        }
        let now = tokio::time::Instant::now();
        let slot = gate.next_slot.filter(|slot| *slot > now).unwrap_or(now);
        if self.config.esi_max_rps > 0 {
            gate.next_slot =
                Some(slot + Duration::from_secs_f64(1.0 / f64::from(self.config.esi_max_rps)));
        }
        Ok(tokio::time::sleep_until(slot))
    }

    fn read_error_limit(&self, headers: &reqwest::header::HeaderMap) {
        let number = |name: &str| {
            headers
                .get(name)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.trim().parse::<i64>().ok())
        };
        let (Some(remain), Some(reset)) = (
            number("x-esi-error-limit-remain"),
            number("x-esi-error-limit-reset"),
        ) else {
            return;
        };
        if remain < ESI_ERROR_LIMIT_FLOOR {
            tracing::warn!(
                remain,
                reset,
                "ESI error limit nearly used up, pausing ESI calls"
            );
            self.gate.lock().expect("ESI gate").closed_until =
                Some(self.clock.now() + time::Duration::seconds(reset.max(1)));
        }
    }
}

async fn read(response: reqwest::Response) -> Result<Answer, EveError> {
    let status = response.status().as_u16();
    let body = response
        .bytes()
        .await
        .map_err(|error| EveError::Transport(error.to_string()))?;
    Ok(Answer {
        status,
        body: body.to_vec(),
    })
}
