//! The single integration-test binary: every suite lives here as a module
//! so a full `cargo test` links one executable.
//!
//! The suites assume they never run concurrently with each other: they
//! truncate the shared tables. `.cargo/config.toml` pins
//! `RUST_TEST_THREADS=1` to keep that assumption true.
//!
//! The ported PHP tests keep their file names (`character_preview`,
//! `demo`, `example`, `killmail_feed`, `process_killmail`,
//! `r2z2_killmail`, `ship_type_service`, `user_agent`); the `golden_*`
//! suites compare with what the Laravel reference answered; the rest
//! cover what the PHP suite never did.

mod common;

mod character_page;
mod character_preview;
mod demo;
mod esi;
mod example;
mod golden_http;
mod golden_jobs;
mod golden_pages;
mod harness;
mod jobs;
mod killmail_feed;
mod listener;
mod options;
mod process_killmail;
mod r2z2_killmail;
mod scheduler;
mod ship_type_service;
mod user_agent;
mod zkillboard;
