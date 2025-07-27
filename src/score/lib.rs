use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{MySqlPool, Row};
use crate::score::*;
use crate::common::ResponseHeader;

pub fn routes() -> Router<MySqlPool> {
    Router::new()
        .route("/:sid/score/begin", post(score_begin))
        .route("/:sid/score/submit", post(score_submit))
}

async fn score_begin(
    State(pool): State<MySqlPool>,
    user: AuthUser,
    Json(request): Json<ScoreBeginRequest>,
) -> Result<Json<ScoreBeginResponse>, AppError> {
    let song_exists = sqlx::query("SELECT 1 FROM song_master WHERE sid = ?")
        .bind(request.score_id)
        .fetch_optional(&pool)
        .await?;

    if song_exists.is_none() {
        return Err(AppError::BadRequest("指定された曲が見つかりません".into()));
    }

    let session_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let mut transaction= pool.begin().await?;

    sqlx::query("INSERT INTO score_session (session_id, score_id, userid) VALUES (?, ?, ?)")
        .bind(&session_id)
        .bind(&request.score_id)
        .bind(&user.userid)
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    Ok(Json(ScoreBeginResponse {
        header: ResponseHeader {
            status: StatusCode::OK.as_u16(),
            master: None,
        },
        session_id: session_id,
    }))
}

async fn score_submit(
    State(pool): State<MySqlPool>,
    user: AuthUser,
    Json(request): Json<ScoreSubmitRequest>,
) -> Result<Json<ScoreSubmitResponse>, AppError> {
    let mut transaction = pool.begin().await?;

    let row = sqlx::query(
        "SELECT score_id FROM score_session
        WHERE session_id = ? AND userid = ?
        AND created_at > NOW() - INTERVAL 60 MINUTE"
    )
    .bind(&request.session_id)
    .bind(&user.userid)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(AppError::NotFound)?;

    let score_id: i32 = row.get("score_id");

    let base_score_row = sqlx::query("SELECT score FROM song_base_score_master LIMIT 1")
        .fetch_one(&mut *transaction)
        .await?;
    let base_score: i32 = base_score_row.get("score");
    if request.score > base_score {
        return Err(AppError::BadRequest("不正なスコア値です".into()));
    }

    sqlx::query(r#"
        INSERT INTO score (userid, score_id, score)
        VALUES (?, ?, ?)
        ON DUPLICATE KEY UPDATE score = GREATEST(score, VALUES(score))
    "#)
    .bind(&user.userid)
    .bind(score_id)
    .bind(request.score)
    .execute(&mut *transaction)
    .await?;

    sqlx::query("DELETE FROM score_session WHERE session_id = ?")
        .bind(&request.session_id)
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    Ok(Json(ScoreSubmitResponse {
        header: ResponseHeader {
            status: StatusCode::OK.as_u16(),
            master: None,
        },
    }))
}
