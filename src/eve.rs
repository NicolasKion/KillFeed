//! The EVE services the app calls: ESI, zKillboard and the R2Z2 firehose.

use crate::clock::Clock;
use crate::config::EveConfig;

/// The answer of an outgoing call: its status and body.
#[derive(Debug, Clone)]
pub struct Answer {
    pub status: u16,
    pub body: Vec<u8>,
}

impl Answer {
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    pub fn json(&self) -> Result<serde_json::Value, EveError> {
        serde_json::from_slice(&self.body).map_err(|error| EveError::Parse(error.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EveError {
    Disabled,
    Transport(String),
    ErrorLimit,
    Parse(String),
}

impl std::fmt::Display for EveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Clone)]
pub struct Eve {}

impl Eve {
    pub fn new(_config: &EveConfig, _clock: Clock) -> Self {
        Self {}
    }

    pub async fn esi_get(&self, _path: &str) -> Result<Answer, EveError> {
        todo!()
    }

    pub async fn zkillboard_get(&self, _path: &str) -> Result<Answer, EveError> {
        todo!()
    }

    pub async fn r2z2_get(&self, _path: &str) -> Result<Answer, EveError> {
        todo!()
    }
}
