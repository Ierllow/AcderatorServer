use axum::Router;
use std::net::SocketAddr;
use sqlx::mysql::MySqlPoolOptions;
use dotenvy::dotenv;
use std::env;

mod score;
mod auth;
mod master;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    master::lib::sync_masters_all(&pool).await.expect("Failed to sync master data");

    let app = Router::new()
        .merge(auth::lib::routes())
        .merge(score::lib::routes())
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
