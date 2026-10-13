//! Seeds the local preview's sample data (`killfeed::demo`): migrates
//! first, then replaces the sample rows. Never run against production.

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let pool = killfeed::db::connect().await.expect("database connection");
    killfeed::db::migrate(&pool)
        .await
        .expect("database migrations");
    let killmails = killfeed::demo::seed(&pool, killfeed::clock::Clock::system().now())
        .await
        .expect("sample data");
    println!(
        "Seeded {killmails} sample killmails; the sample overlay is /{}",
        killfeed::demo::DEMO_CHARACTER_ID
    );
}
