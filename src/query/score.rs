use crate::common::AppError;
use sqlx::{MySql, MySqlPool, QueryBuilder, Row, Transaction};

pub struct ScoreRecord {
    pub score_id: i32,
    pub score: i32,
}

pub enum ScoreFilter<'a> {
    Userid(&'a str),
}

pub struct ScoreQuery<'a> {
    pool: &'a MySqlPool,
    filters: Vec<ScoreFilter<'a>>,
}

impl<'a> ScoreQuery<'a> {
    pub fn new(pool: &'a MySqlPool) -> Self {
        Self {
            pool,
            filters: Vec::new(),
        }
    }

    pub fn filter(mut self, filter: ScoreFilter<'a>) -> Self {
        self.filters.push(filter);
        self
    }

    pub async fn upsert_best(
        transaction: &mut Transaction<'_, MySql>,
        userid: &str,
        score_id: i32,
        score: i32,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO score (userid, score_id, score)
            VALUES (?, ?, ?)
            ON DUPLICATE KEY UPDATE score = GREATEST(score, VALUES(score))
        "#,
        )
        .bind(userid)
        .bind(score_id)
        .bind(score)
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn any(self) -> Result<bool, AppError> {
        let mut query = QueryBuilder::new("SELECT 1 FROM score");
        self.push_where(&mut query);
        query.push(" LIMIT 1");

        Ok(query.build().fetch_optional(self.pool).await?.is_some())
    }

    pub async fn first(self) -> Result<Option<ScoreRecord>, AppError> {
        self.one("submitted_at ASC").await
    }

    pub async fn last(self) -> Result<Option<ScoreRecord>, AppError> {
        self.one("submitted_at DESC").await
    }

    pub async fn all(self) -> Result<Vec<ScoreRecord>, AppError> {
        let mut query = QueryBuilder::new("SELECT score_id, score FROM score");
        self.push_where(&mut query);
        query.push(" ORDER BY score_id ASC");

        let rows = query.build().fetch_all(self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|row| ScoreRecord {
                score_id: row.get("score_id"),
                score: row.get("score"),
            })
            .collect())
    }

    async fn one(self, order_by: &str) -> Result<Option<ScoreRecord>, AppError> {
        let mut query = QueryBuilder::new("SELECT score_id, score FROM score");
        self.push_where(&mut query);
        query.push(" ORDER BY ");
        query.push(order_by);
        query.push(" LIMIT 1");

        let row = query.build().fetch_optional(self.pool).await?;
        Ok(row.map(|row| ScoreRecord {
            score_id: row.get("score_id"),
            score: row.get("score"),
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
                ScoreFilter::Userid(userid) => {
                    query.push("userid = ");
                    query.push_bind(*userid);
                }
            }
        }
    }
}
