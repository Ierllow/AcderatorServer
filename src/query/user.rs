use crate::common::AppError;
use sqlx::{MySql, MySqlPool, QueryBuilder, Row, Transaction};

pub struct UserRecord {
    pub userid: String,
    pub password: String,
}

pub enum UserFilter<'a> {
    Userid(&'a str),
    Uuid(&'a str),
}

pub struct UserQuery<'a> {
    pool: &'a MySqlPool,
    filters: Vec<UserFilter<'a>>,
}

impl<'a> UserQuery<'a> {
    pub fn new(pool: &'a MySqlPool) -> Self {
        Self {
            pool,
            filters: Vec::new(),
        }
    }

    pub fn filter(mut self, filter: UserFilter<'a>) -> Self {
        self.filters.push(filter);
        self
    }

    pub async fn create(
        transaction: &mut Transaction<'_, MySql>,
        userid: u32,
        uuid: &str,
        password: u32,
    ) -> Result<(), AppError> {
        sqlx::query("INSERT INTO user (userid, uuid, password) VALUES (?, ?, ?)")
            .bind(userid)
            .bind(uuid)
            .bind(password)
            .execute(&mut **transaction)
            .await?;

        Ok(())
    }

    pub async fn any(self) -> Result<bool, AppError> {
        let mut query = QueryBuilder::new("SELECT 1 FROM user");
        self.push_where(&mut query);
        query.push(" LIMIT 1");

        Ok(query.build().fetch_optional(self.pool).await?.is_some())
    }

    pub async fn first(self) -> Result<Option<UserRecord>, AppError> {
        self.one("userid ASC").await
    }

    pub async fn last(self) -> Result<Option<UserRecord>, AppError> {
        self.one("userid DESC").await
    }

    async fn one(self, order_by: &str) -> Result<Option<UserRecord>, AppError> {
        let mut query = QueryBuilder::new("SELECT userid, password FROM user");
        self.push_where(&mut query);
        query.push(" ORDER BY ");
        query.push(order_by);
        query.push(" LIMIT 1");

        let row = query.build().fetch_optional(self.pool).await?;
        Ok(row.map(|row| UserRecord {
            userid: row.get("userid"),
            password: row.get("password"),
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
                UserFilter::Userid(userid) => {
                    query.push("userid = ");
                    query.push_bind(*userid);
                }
                UserFilter::Uuid(uuid) => {
                    query.push("uuid = ");
                    query.push_bind(*uuid);
                }
            }
        }
    }
}
