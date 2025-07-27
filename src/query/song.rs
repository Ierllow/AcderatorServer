use crate::common::AppError;
use sqlx::{MySqlPool, QueryBuilder, Row};

pub struct SongUpsert {
    pub sid: i32,
    pub group: i32,
    pub difficulty: i32,
    pub name: String,
    pub composer: String,
    pub start_offset: f32,
    pub bg: i32,
}

pub struct SongRecord {
    pub sid: i32,
    pub group: i32,
    pub difficulty: i32,
    pub name: String,
    pub composer: String,
    pub start_offset: f32,
    pub bg: i32,
}

pub enum SongFilter {
    Sid(i32),
}

pub struct SongQuery<'a> {
    pool: &'a MySqlPool,
    filters: Vec<SongFilter>,
}

impl<'a> SongQuery<'a> {
    pub fn new(pool: &'a MySqlPool) -> Self {
        Self {
            pool,
            filters: Vec::new(),
        }
    }

    pub fn filter(mut self, filter: SongFilter) -> Self {
        self.filters.push(filter);
        self
    }

    pub async fn upsert(pool: &MySqlPool, song: SongUpsert) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO song_master (sid, `group`, difficulty, name, composer, start_offset, bg)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE
               `group`=VALUES(`group`),
               difficulty=VALUES(difficulty),
               name=VALUES(name),
               composer=VALUES(composer),
               start_offset=VALUES(start_offset),
               bg=VALUES(bg)",
        )
        .bind(song.sid)
        .bind(song.group)
        .bind(song.difficulty)
        .bind(song.name)
        .bind(song.composer)
        .bind(song.start_offset)
        .bind(song.bg)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn any(self) -> Result<bool, AppError> {
        let mut query = QueryBuilder::new("SELECT 1 FROM song_master");
        self.push_where(&mut query);
        query.push(" LIMIT 1");

        Ok(query.build().fetch_optional(self.pool).await?.is_some())
    }

    pub async fn first(self) -> Result<Option<SongRecord>, AppError> {
        self.one("sid ASC").await
    }

    pub async fn last(self) -> Result<Option<SongRecord>, AppError> {
        self.one("sid DESC").await
    }

    pub async fn all(self) -> Result<Vec<SongRecord>, AppError> {
        let mut query = QueryBuilder::new(
            "SELECT sid, `group`, difficulty, name, composer, start_offset, bg FROM song_master",
        );
        self.push_where(&mut query);
        query.push(" ORDER BY sid ASC");

        let rows = query.build().fetch_all(self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|row| SongRecord {
                sid: row.get("sid"),
                group: row.get("group"),
                difficulty: row.get("difficulty"),
                name: row.get("name"),
                composer: row.get("composer"),
                start_offset: row.get("start_offset"),
                bg: row.get("bg"),
            })
            .collect())
    }

    async fn one(self, order_by: &str) -> Result<Option<SongRecord>, AppError> {
        let mut query = QueryBuilder::new(
            "SELECT sid, `group`, difficulty, name, composer, start_offset, bg FROM song_master",
        );
        self.push_where(&mut query);
        query.push(" ORDER BY ");
        query.push(order_by);
        query.push(" LIMIT 1");

        let row = query.build().fetch_optional(self.pool).await?;
        Ok(row.map(|row| SongRecord {
            sid: row.get("sid"),
            group: row.get("group"),
            difficulty: row.get("difficulty"),
            name: row.get("name"),
            composer: row.get("composer"),
            start_offset: row.get("start_offset"),
            bg: row.get("bg"),
        }))
    }

    fn push_where(&self, query: &mut QueryBuilder<'_, sqlx::MySql>) {
        if self.filters.is_empty() {
            return;
        }

        query.push(" WHERE ");
        for (i, filter) in self.filters.iter().enumerate() {
            if i > 0 {
                query.push(" AND ");
            }

            match filter {
                SongFilter::Sid(sid) => {
                    query.push("sid = ");
                    query.push_bind(*sid);
                }
            }
        }
    }
}
