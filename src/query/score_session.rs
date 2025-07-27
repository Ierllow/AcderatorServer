use crate::common::AppError;
use sqlx::{MySql, MySqlPool, QueryBuilder, Row, Transaction};

pub struct ScoreSessionRecord {
    pub score_id: i32,
    pub userid: String,
}

pub enum ScoreSessionFilter<'a> {
    SessionId(&'a str),
    ActiveWithinMinutes(i32),
}

pub struct ScoreSessionQuery<'a> {
    pool: &'a MySqlPool,
    filters: Vec<ScoreSessionFilter<'a>>,
}

impl<'a> ScoreSessionQuery<'a> {
    pub fn new(pool: &'a MySqlPool) -> Self {
        Self {
            pool,
            filters: Vec::new(),
        }
    }

    pub fn filter(mut self, filter: ScoreSessionFilter<'a>) -> Self {
        self.filters.push(filter);
        self
    }

    pub async fn create(
        transaction: &mut Transaction<'_, MySql>,
        session_id: &str,
        score_id: i32,
        userid: &str,
    ) -> Result<(), AppError> {
        sqlx::query("INSERT INTO score_session (session_id, score_id, userid) VALUES (?, ?, ?)")
            .bind(session_id)
            .bind(score_id)
            .bind(userid)
            .execute(&mut **transaction)
            .await?;

        Ok(())
    }

    pub async fn delete_by_session_id(
        transaction: &mut Transaction<'_, MySql>,
        session_id: &str,
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM score_session WHERE session_id = ?")
            .bind(session_id)
            .execute(&mut **transaction)
            .await?;

        Ok(())
    }

    pub async fn any(self) -> Result<bool, AppError> {
        let mut query = QueryBuilder::new("SELECT 1 FROM score_session");
        self.push_where(&mut query);
        query.push(" LIMIT 1");

        Ok(query.build().fetch_optional(self.pool).await?.is_some())
    }

    pub async fn first(self) -> Result<Option<ScoreSessionRecord>, AppError> {
        self.one("created_at ASC").await
    }

    pub async fn last(self) -> Result<Option<ScoreSessionRecord>, AppError> {
        self.one("created_at DESC").await
    }

    async fn one(self, order_by: &str) -> Result<Option<ScoreSessionRecord>, AppError> {
        let mut query = QueryBuilder::new("SELECT score_id, userid FROM score_session");
        self.push_where(&mut query);
        query.push(" ORDER BY ");
        query.push(order_by);
        query.push(" LIMIT 1");

        let row = query.build().fetch_optional(self.pool).await?;
        Ok(row.map(|row| ScoreSessionRecord {
            score_id: row.get("score_id"),
            userid: row.get("userid"),
        }))
    }

    fn push_where(&self, query: &mut QueryBuilder<'a, sqlx::MySql>) {
        if self.filters.is_empty() {
            return;
        }

        query.push(" WHERE ");
        for (i, filter) in self.filters.iter().enumerate() {
            if i > 0 {
                query.push(" AND ");
            }

            match filter {
                ScoreSessionFilter::SessionId(session_id) => {
                    query.push("session_id = ");
                    query.push_bind(*session_id);
                }
                ScoreSessionFilter::ActiveWithinMinutes(minutes) => {
                    query.push("TIMESTAMPDIFF(MINUTE, created_at, NOW()) < ");
                    query.push_bind(*minutes);
                }
            }
        }
    }
}
