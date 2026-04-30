#[cfg(feature = "debug-ui")]
use acderator_sv::debug;
use acderator_sv::{
    auth,
    common::{self, AppState, RateLimiter},
    master, score, user,
};
use axum::{middleware, Router};
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("failed to connect to database");

    master::lib::bulk_insert_masters(&pool)
        .await
        .expect("failed to sync master data");

    let state = AppState {
        pool,
        rate_limiter: RateLimiter::new(),
    };

    let app = Router::new()
        .merge(auth::api::routes())
        .merge(score::api::routes())
        .merge(user::api::routes());

    #[cfg(feature = "debug-ui")]
    let app = app.merge(debug::api::routes());

    let app = app
        .layer(middleware::from_fn_with_state(
            state.clone(),
            common::rate_limit_guard,
        ))
        .layer(middleware::from_fn(common::maintenance_guard))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
