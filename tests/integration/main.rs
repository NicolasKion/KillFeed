//! The single integration-test binary: every suite lives here as a module
//! so a full `cargo test` links one executable.
//!
//! The suites assume they never run concurrently with each other: they
//! truncate the shared tables. `.cargo/config.toml` pins
//! `RUST_TEST_THREADS=1` to keep that assumption true.

mod common;

mod character_page;
mod character_preview;
mod demo;
mod demo_seed;
mod esi;
mod example;
mod jobs;
mod killmail_feed;
mod listener;
mod options;
mod pages;
mod process_killmail;
mod r2z2_killmail;
mod routes;
mod scheduler;
mod ship_type_service;
mod user_agent;
mod zkillboard;
