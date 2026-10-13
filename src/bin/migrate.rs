//! Applies the database migrations and exits, so the schema can be in place
//! before the api first starts (a data import, for one).

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let pool = killfeed::db::connect().await.expect("database connection");
    killfeed::db::migrate(&pool)
        .await
        .expect("database migrations");
    println!("Migrations applied.");
}
