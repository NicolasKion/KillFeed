//! Deletes the killmails older
//! than KILLMAIL_MAX_AGE_HOURS now (the api's scheduler runs it every
//! night at 00:00 UTC).

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = killfeed::config::Config::from_env();
    let pool = killfeed::db::connect().await.expect("database connection");
    let hours = config.max_killmail_age_hours;
    let deleted =
        killfeed::purge::purge_old_killmails(&pool, killfeed::clock::Clock::system().now(), hours)
            .await
            .expect("old killmails purge");
    println!("{}", killfeed::purge::report(deleted, hours));
}
