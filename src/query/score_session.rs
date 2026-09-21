use crate::common::AppError;
use sqlx::{MySql, Row, Transaction};

pub struct ScoreSessionRecord {
    pub score_id: i32,
}

pub struct ScoreSessionQuery;

impl ScoreSessionQuery {
    pub async fn create(
        transaction: &mut Transaction<'_, MySql>,
        session_hash: &str,
        score_id: i32,
        userid: &str,
    ) -> Result<(), AppError> {
        sqlx::query("INSERT INTO score_session (session_id, score_id, userid) VALUES (?, ?, ?)")
            .bind(session_hash)
            .bind(score_id)
            .bind(userid)
            .execute(&mut **transaction)
            .await?;
        Ok(())
    }

    pub async fn invalidate_previous(
        transaction: &mut Transaction<'_, MySql>,
        userid: &str,
        score_id: i32,
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM score_session WHERE userid = ? AND score_id = ?")
            .bind(userid)
            .bind(score_id)
            .execute(&mut **transaction)
            .await?;
        Ok(())
    }

    pub async fn consume(
        transaction: &mut Transaction<'_, MySql>,
        session_hash: &str,
        userid: &str,
        ttl_seconds: u32,
    ) -> Result<Option<ScoreSessionRecord>, AppError> {
        let row = sqlx::query(
            "SELECT score_id
             FROM score_session
             WHERE session_id = ?
               AND userid = ?
               AND created_at >= DATE_SUB(NOW(), INTERVAL ? SECOND)
             LIMIT 1
             FOR UPDATE",
        )
        .bind(session_hash)
        .bind(userid)
        .bind(ttl_seconds)
        .fetch_optional(&mut **transaction)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        sqlx::query("DELETE FROM score_session WHERE session_id = ?")
            .bind(session_hash)
            .execute(&mut **transaction)
            .await?;

        Ok(Some(ScoreSessionRecord {
            score_id: row.get("score_id"),
        }))
    }

    pub async fn prune_expired(
        transaction: &mut Transaction<'_, MySql>,
        ttl_seconds: u32,
    ) -> Result<(), AppError> {
        sqlx::query(
            "DELETE FROM score_session
             WHERE created_at < DATE_SUB(NOW(), INTERVAL ? SECOND)",
        )
        .bind(ttl_seconds)
        .execute(&mut **transaction)
        .await?;
        Ok(())
    }
}
