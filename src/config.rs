//! Runtime configuration from the environment.

use std::time::Duration;

/// Everything the app reads from the environment, resolved once at
/// startup. Tests build one by hand instead of mutating process env vars.
#[derive(Debug, Clone)]
pub struct Config {
    /// `APP_NAME`: the health page title.
    pub app_name: String,
    /// `KILLMAIL_MAX_VISIBLE`: the most cards an overlay asks for (and the
    /// default count), at least 1.
    pub max_visible_killmails: i64,
    /// `KILLMAIL_MAX_AGE_HOURS`: the longest time window an overlay can
    /// ask for (and the default), and the age the nightly purge deletes
    /// at; at least 1.
    pub max_killmail_age_hours: i64,
    /// `KILLFEED_DISCORD_INVITE_URL`: the Discord link of the setup page.
    pub discord_invite_url: Option<String>,
    pub eve: EveConfig,
    pub background: BackgroundConfig,
}

/// The EVE services the app calls: ESI, zKillboard and its R2Z2
/// firehose. The base URLs point at local mock servers in the tests.
#[derive(Debug, Clone)]
pub struct EveConfig {
    /// `EVE_APIS_ENABLED`: off, every outgoing call fails at once without
    /// touching the network (the dev stack's default), so the app behaves
    /// as with ESI down: unknown characters are not found, meta groups
    /// fall back to Tech I, overlays render from stored data.
    pub enabled: bool,
    /// `ESI_BASE_URL`, the unversioned ESI (`https://esi.evetech.net`).
    pub esi_base_url: String,
    /// `ZKILLBOARD_BASE_URL` (`https://zkillboard.com`).
    pub zkillboard_base_url: String,
    /// `R2Z2_BASE_URL`, the firehose bucket
    /// (`https://r2z2.zkillboard.com/ephemeral`).
    pub r2z2_base_url: String,
    /// `R2Z2_MAX_CONSECUTIVE_404S`: missing sequences in a row before the
    /// listener asks `sequence.json` where the firehose is.
    pub r2z2_max_consecutive_404s: u32,
    /// `HTTP_USER_AGENT`: sent with every outgoing request.
    pub user_agent: String,
    /// `ESI_MAX_RPS`: requests per second to ESI at most; 0 is unlimited.
    pub esi_max_rps: u32,
    /// The pauses between retries; tests
    /// shorten them.
    pub timing: RetryTiming,
}

#[derive(Debug, Clone)]
pub struct RetryTiming {
    /// Between the five tries of a killmail fetch from ESI (1 s).
    pub esi_killmail_retry: Duration,
    /// Between the three tries of a type lookup on ESI (500 ms).
    pub esi_type_retry: Duration,
    /// Between the listener's items (100 ms).
    pub listener_item_pause: Duration,
    /// After a missing or failed sequence (6 s).
    pub listener_retry_pause: Duration,
}

impl Default for RetryTiming {
    fn default() -> Self {
        Self {
            esi_killmail_retry: Duration::from_millis(1000),
            esi_type_retry: Duration::from_millis(500),
            listener_item_pause: Duration::from_millis(100),
            listener_retry_pause: Duration::from_secs(6),
        }
    }
}

/// What runs beside the requests.
#[derive(Debug, Clone)]
pub struct BackgroundConfig {
    /// `LISTENER_ENABLED`: the R2Z2 firehose listener (off by default:
    /// two listeners at once double the load on R2Z2).
    pub listener: bool,
    /// `SCHEDULER_ENABLED`: the nightly purge and the housekeeping (on by
    /// default).
    pub scheduler: bool,
    /// `JOB_WORKERS`: queue workers processing killmails; 0 runs none.
    pub job_workers: usize,
}

pub const DEFAULT_APP_NAME: &str = "Killfeed";
pub const DEFAULT_MAX_VISIBLE_KILLMAILS: i64 = 50;
pub const DEFAULT_MAX_KILLMAIL_AGE_HOURS: i64 = 168;
pub const DEFAULT_ESI_BASE_URL: &str = "https://esi.evetech.net";
pub const DEFAULT_ZKILLBOARD_BASE_URL: &str = "https://zkillboard.com";
pub const DEFAULT_R2Z2_BASE_URL: &str = "https://r2z2.zkillboard.com/ephemeral";
pub const DEFAULT_R2Z2_MAX_CONSECUTIVE_404S: u32 = 10;
pub const DEFAULT_USER_AGENT: &str =
    "KillFeed/1.0 (https://killfeed.killmail.space; nicolaskion07@gmail.com; EVE: Nicolas Kion)";
pub const DEFAULT_ESI_MAX_RPS: u32 = 20;
pub const DEFAULT_JOB_WORKERS: usize = 4;

/// A switch: `true`, `1`, `on` or `yes`; anything else is off.
fn env_flag(key: &str, default: bool) -> bool {
    env(key).map_or(default, |value| {
        matches!(
            value.to_ascii_lowercase().as_str(),
            "true" | "1" | "on" | "yes"
        )
    })
}

fn env_or(key: &str, default: &str) -> String {
    env(key).unwrap_or_else(|| default.to_owned())
}

/// A variable that is set and not empty (`null` counts as unset).
fn env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty() && value != "null")
}

/// A number from the environment; one that does not parse stops the
/// startup instead of running with a surprise.
fn env_number<T: std::str::FromStr>(key: &str, default: T) -> T {
    env(key).map_or(default, |value| {
        value
            .parse()
            .unwrap_or_else(|_| panic!("{key} is {value:?}, expected a number"))
    })
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            app_name: env_or("APP_NAME", DEFAULT_APP_NAME),
            max_visible_killmails: env_number(
                "KILLMAIL_MAX_VISIBLE",
                DEFAULT_MAX_VISIBLE_KILLMAILS,
            )
            .max(1),
            max_killmail_age_hours: env_number(
                "KILLMAIL_MAX_AGE_HOURS",
                DEFAULT_MAX_KILLMAIL_AGE_HOURS,
            )
            .max(1),
            discord_invite_url: env("KILLFEED_DISCORD_INVITE_URL"),
            eve: EveConfig {
                enabled: env_flag("EVE_APIS_ENABLED", true),
                esi_base_url: trim_url(env_or("ESI_BASE_URL", DEFAULT_ESI_BASE_URL)),
                zkillboard_base_url: trim_url(env_or(
                    "ZKILLBOARD_BASE_URL",
                    DEFAULT_ZKILLBOARD_BASE_URL,
                )),
                r2z2_base_url: trim_url(env_or("R2Z2_BASE_URL", DEFAULT_R2Z2_BASE_URL)),
                r2z2_max_consecutive_404s: env_number(
                    "R2Z2_MAX_CONSECUTIVE_404S",
                    DEFAULT_R2Z2_MAX_CONSECUTIVE_404S,
                )
                .max(1),
                user_agent: env_or("HTTP_USER_AGENT", DEFAULT_USER_AGENT),
                esi_max_rps: env_number("ESI_MAX_RPS", DEFAULT_ESI_MAX_RPS),
                timing: RetryTiming::default(),
            },
            background: BackgroundConfig {
                listener: env_flag("LISTENER_ENABLED", false),
                scheduler: env_flag("SCHEDULER_ENABLED", true),
                job_workers: env_number("JOB_WORKERS", DEFAULT_JOB_WORKERS),
            },
        }
    }
}

fn trim_url(url: String) -> String {
    url.trim_end_matches('/').to_owned()
}
