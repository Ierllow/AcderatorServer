use crate::common::AppError;
use sqlx::{MySql, MySqlPool, QueryBuilder, Row, Transaction};

pub struct SessionRecord {
    pub userid: String,
}

pub enum SessionFilter<'a> {
    SessionId(&'a str),
    ActiveWithinMinutes(i32),
}

pub struct SessionQuery<'a> {
    pool: &'a MySqlPool,
    filters: Vec<SessionFilter<'a>>,
}

impl<'a> SessionQuery<'a> {
    pub fn new(pool: &'a MySqlPool) -> Self {
        Self {
            pool,
            filters: Vec::new(),
        }
    }

    pub fn filter(mut self, filter: SessionFilter<'a>) -> Self {
        self.filters.push(filter);
        self
    }

    pub async fn create(
        transaction: &mut Transaction<'_, MySql>,
        session_id: &str,
        userid: impl ToString,
    ) -> Result<(), AppError> {
        sqlx::query("INSERT INTO session (session_id, userid, last_activity) VALUES (?, ?, NOW())")
            .bind(session_id)
            .bind(userid.to_string())
            .execute(&mut **transaction)
            .await?;

        Ok(())
    }

    pub async fn touch(pool: &MySqlPool, session_id: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE session SET last_activity = NOW() WHERE session_id = ?")
            .bind(session_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn any(self) -> Result<bool, AppError> {
        let mut query = QueryBuilder::new("SELECT 1 FROM session");
        self.push_where(&mut query);
        query.push(" LIMIT 1");

        Ok(query.build().fetch_optional(self.pool).await?.is_some())
    }

    pub async fn first(self) -> Result<Option<SessionRecord>, AppError> {
        self.one("last_activity ASC").await
    }

    pub async fn last(self) -> Result<Option<SessionRecord>, AppError> {
        self.one("last_activity DESC").await
    }

    async fn one(self, order_by: &str) -> Result<Option<SessionRecord>, AppError> {
        let mut query = QueryBuilder::new("SELECT userid FROM session");
        self.push_where(&mut query);
        query.push(" ORDER BY ");
        query.push(order_by);
        query.push(" LIMIT 1");

        let row = query.build().fetch_optional(self.pool).await?;
        Ok(row.map(|row| SessionRecord {
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
                SessionFilter::SessionId(session_id) => {
                    query.push("session_id = ");
                    query.push_bind(*session_id);
                }
                SessionFilter::ActiveWithinMinutes(minutes) => {
                    query.push("TIMESTAMPDIFF(MINUTE, last_activity, NOW()) < ");
                    query.push_bind(*minutes);
                }
            }
        }
    }
}
