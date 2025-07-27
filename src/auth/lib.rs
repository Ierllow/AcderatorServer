use axum::{extract::State, Json, Router, routing::post};
use sqlx::MySqlPool;
use uuid::Uuid;
use crate::auth::{AuthRequest, AuthResponse};
use crate::utils::AppError;

pub fn routes() -> Router<MySqlPool> {
    Router::new().route("/auth/login", post(login))
}

async fn login(
    State(pool): State<MySqlPool>,
    Json(request): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let userid = request.userid;

    let user = sqlx::query("SELECT * FROM users WHERE userid = ?")
        .bind(&userid)
        .fetch_optional(&pool)
        .await?;

    if user.is_none() {
        sqlx::query("INSERT INTO users (userid, password) VALUES (?, '')")
            .bind(&userid)
            .execute(&pool)
            .await?;
    }

    let session_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO sessions (session_id, userid) VALUES (?, ?)")
        .bind(&session_id)
        .bind(&userid)
        .execute(&pool)
        .await?;

    Ok(Json(AuthResponse { token: session_id }))
}
