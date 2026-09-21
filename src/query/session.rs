use crate::common::AppError;
use sqlx::{MySql, MySqlPool, Row, Transaction};

pub struct SessionRecord {
    pub userid: String,
}

pub struct SessionQuery;

impl SessionQuery {
    pub async fn create(
        transaction: &mut Transaction<'_, MySql>,
        session_hash: &str,
        userid: impl ToString,
    ) -> Result<(), AppError> {
        sqlx::query("INSERT INTO session (session_id, userid, last_activity) VALUES (?, ?, NOW())")
            .bind(session_hash)
            .bind(userid.to_string())
            .execute(&mut **transaction)
            .await?;
        Ok(())
    }

    pub async fn authenticate(
        pool: &MySqlPool,
        session_hash: &str,
        ttl_seconds: u32,
    ) -> Result<Option<SessionRecord>, AppError> {
        let mut transaction = pool.begin().await?;
        let update = sqlx::query(
            "UPDATE session
             SET last_activity = NOW()
             WHERE session_id = ?
               AND last_activity >= DATE_SUB(NOW(), INTERVAL ? SECOND)",
        )
        .bind(session_hash)
        .bind(ttl_seconds)
        .execute(&mut *transaction)
        .await?;

        if update.rows_affected() != 1 {
            return Ok(None);
        }

        let row = sqlx::query("SELECT userid FROM session WHERE session_id = ? LIMIT 1")
            .bind(session_hash)
            .fetch_optional(&mut *transaction)
            .await?;
        transaction.commit().await?;

        Ok(row.map(|row| SessionRecord {
            userid: row.get("userid"),
        }))
    }

    pub async fn prune_expired(
        transaction: &mut Transaction<'_, MySql>,
        ttl_seconds: u32,
    ) -> Result<(), AppError> {
        sqlx::query(
            "DELETE FROM session
             WHERE last_activity < DATE_SUB(NOW(), INTERVAL ? SECOND)",
        )
        .bind(ttl_seconds)
        .execute(&mut **transaction)
        .await?;
        Ok(())
    }

    pub async fn delete(
        transaction: &mut Transaction<'_, MySql>,
        session_hash: &str,
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM session WHERE session_id = ?")
            .bind(session_hash)
            .execute(&mut **transaction)
            .await?;
        Ok(())
    }
}
