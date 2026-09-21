use crate::common::AppError;
use crate::master::MasterDataResponse;
use sqlx::{MySql, MySqlPool, QueryBuilder, Row, Transaction};

pub struct MasterVersionQuery<'a> {
    pool: &'a MySqlPool,
}

impl<'a> MasterVersionQuery<'a> {
    pub fn new(pool: &'a MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn any(self) -> Result<bool, AppError> {
        Ok(sqlx::query("SELECT 1 FROM master_version LIMIT 1")
            .fetch_optional(self.pool)
            .await?
            .is_some())
    }

    pub async fn first(self) -> Result<Option<String>, AppError> {
        let row = sqlx::query("SELECT version FROM master_version ORDER BY version ASC LIMIT 1")
            .fetch_optional(self.pool)
            .await?;
        Ok(row.map(|row| row.get("version")))
    }

    pub async fn last(self) -> Result<Option<String>, AppError> {
        let row = sqlx::query("SELECT version FROM master_version ORDER BY version DESC LIMIT 1")
            .fetch_optional(self.pool)
            .await?;
        Ok(row.map(|row| row.get("version")))
    }

    pub async fn replace(pool: &MySqlPool, version: &str) -> Result<(), AppError> {
        let mut transaction = pool.begin().await?;
        sqlx::query("DELETE FROM master_version")
            .execute(&mut *transaction)
            .await?;
        sqlx::query("INSERT INTO master_version (version) VALUES (?)")
            .bind(version)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(())
    }
}

pub struct BaseScoreQuery<'a> {
    pool: &'a MySqlPool,
}

impl<'a> BaseScoreQuery<'a> {
    pub fn new(pool: &'a MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn any(self) -> Result<bool, AppError> {
        Ok(sqlx::query("SELECT 1 FROM song_base_score_master LIMIT 1")
            .fetch_optional(self.pool)
            .await?
            .is_some())
    }

    pub async fn first(self) -> Result<Option<i32>, AppError> {
        let row = sqlx::query("SELECT score FROM song_base_score_master LIMIT 1")
            .fetch_optional(self.pool)
            .await?;
        Ok(row.map(|row| row.get("score")))
    }

    pub async fn last(self) -> Result<Option<i32>, AppError> {
        self.first().await
    }

    pub async fn replace(pool: &MySqlPool, score: i32) -> Result<(), AppError> {
        let mut transaction = pool.begin().await?;
        sqlx::query("DELETE FROM song_base_score_master")
            .execute(&mut *transaction)
            .await?;
        sqlx::query("INSERT INTO song_base_score_master (score) VALUES (?)")
            .bind(score)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(())
    }
}

pub struct MasterDataQuery;

pub struct MasterTableCounts {
    pub master_version: usize,
    pub title_masters: usize,
    pub song_select_masters: usize,
    pub song_masters: usize,
    pub score_rate_masters: usize,
    pub base_score_masters: usize,
    pub judge_zone_masters: usize,
    pub base_hp_masters: usize,
    pub hp_rate_masters: usize,
    pub sound_sheet_masters: usize,
    pub result_masters: usize,
}

impl MasterDataQuery {
    pub async fn counts(pool: &MySqlPool) -> Result<MasterTableCounts, AppError> {
        let row = sqlx::query(
            "SELECT
                (SELECT COUNT(*) FROM master_version) AS master_version,
                (SELECT COUNT(*) FROM title_master) AS title_masters,
                (SELECT COUNT(*) FROM song_select_master) AS song_select_masters,
                (SELECT COUNT(*) FROM song_master) AS song_masters,
                (SELECT COUNT(*) FROM song_score_rate_master) AS score_rate_masters,
                (SELECT COUNT(*) FROM song_base_score_master) AS base_score_masters,
                (SELECT COUNT(*) FROM song_judge_zone_master) AS judge_zone_masters,
                (SELECT COUNT(*) FROM song_base_hp_master) AS base_hp_masters,
                (SELECT COUNT(*) FROM song_hp_rate_master) AS hp_rate_masters,
                (SELECT COUNT(*) FROM sound_sheet_name_master) AS sound_sheet_masters,
                (SELECT COUNT(*) FROM result_master) AS result_masters",
        )
        .fetch_one(pool)
        .await?;

        Ok(MasterTableCounts {
            master_version: count(&row, "master_version"),
            title_masters: count(&row, "title_masters"),
            song_select_masters: count(&row, "song_select_masters"),
            song_masters: count(&row, "song_masters"),
            score_rate_masters: count(&row, "score_rate_masters"),
            base_score_masters: count(&row, "base_score_masters"),
            judge_zone_masters: count(&row, "judge_zone_masters"),
            base_hp_masters: count(&row, "base_hp_masters"),
            hp_rate_masters: count(&row, "hp_rate_masters"),
            sound_sheet_masters: count(&row, "sound_sheet_masters"),
            result_masters: count(&row, "result_masters"),
        })
    }

    pub async fn replace_all(pool: &MySqlPool, root: &MasterDataResponse) -> Result<(), AppError> {
        let mut transaction = pool.begin().await?;
        clear_master_tables(&mut transaction).await?;

        sqlx::query("INSERT INTO master_version (version) VALUES (?)")
            .bind(&root.version_master)
            .execute(&mut *transaction)
            .await?;

        bulk_insert(
            &mut transaction,
            "INSERT INTO title_master (tid)",
            &root.title_masters,
            |mut b, t| {
                b.push_bind(t.tid);
            },
            "ON DUPLICATE KEY UPDATE tid=tid",
        )
        .await?;

        bulk_insert(
            &mut transaction,
            "INSERT INTO song_select_master (`group`, start_song_time, song_time)",
            &root.song_select_masters,
            |mut b, s| {
                b.push_bind(s.group)
                    .push_bind(s.start_song_time)
                    .push_bind(s.song_time);
            },
            "ON DUPLICATE KEY UPDATE start_song_time=VALUES(start_song_time), song_time=VALUES(song_time)",
        )
        .await?;

        bulk_insert(
            &mut transaction,
            "INSERT INTO song_master (sid, `group`, difficulty, name, composer, start_offset, bg)",
            &root.song_masters,
            |mut b, s| {
                b.push_bind(s.sid)
                    .push_bind(s.group)
                    .push_bind(s.difficulty)
                    .push_bind(&s.name)
                    .push_bind(&s.composer)
                    .push_bind(s.start_offset)
                    .push_bind(s.bg);
            },
            "ON DUPLICATE KEY UPDATE `group`=VALUES(`group`), difficulty=VALUES(difficulty), name=VALUES(name), composer=VALUES(composer), start_offset=VALUES(start_offset), bg=VALUES(bg)",
        )
        .await?;

        bulk_insert(
            &mut transaction,
            "INSERT INTO song_score_rate_master (r_type, rate)",
            &root.score_rate_masters,
            |mut b, r| {
                b.push_bind(r.r_type).push_bind(r.rate);
            },
            "ON DUPLICATE KEY UPDATE rate=VALUES(rate)",
        )
        .await?;

        bulk_insert(
            &mut transaction,
            "INSERT INTO song_judge_zone_master (j_type, zone)",
            &root.judge_zone_masters,
            |mut b, j| {
                b.push_bind(j.j_type).push_bind(j.zone);
            },
            "ON DUPLICATE KEY UPDATE zone=VALUES(zone)",
        )
        .await?;

        bulk_insert(
            &mut transaction,
            "INSERT INTO song_hp_rate_master (j_type, rate)",
            &root.hp_rate_masters,
            |mut b, h| {
                b.push_bind(h.j_type).push_bind(h.rate);
            },
            "ON DUPLICATE KEY UPDATE rate=VALUES(rate)",
        )
        .await?;

        bulk_insert(
            &mut transaction,
            "INSERT INTO sound_sheet_name_master (category, id, sheet_name, cue_name)",
            &root.sound_sheet_masters,
            |mut b, s| {
                b.push_bind(s.category)
                    .push_bind(s.id)
                    .push_bind(&s.sheet_name)
                    .push_bind(&s.cue_name);
            },
            "ON DUPLICATE KEY UPDATE sheet_name=VALUES(sheet_name), cue_name=VALUES(cue_name)",
        )
        .await?;

        bulk_insert(
            &mut transaction,
            "INSERT INTO result_master (rid)",
            &root.result_masters,
            |mut b, r| {
                b.push_bind(r.rid);
            },
            "ON DUPLICATE KEY UPDATE rid=rid",
        )
        .await?;

        if let Some(s) = root.base_score_masters.first() {
            sqlx::query("INSERT INTO song_base_score_master (score) VALUES (?)")
                .bind(s.score)
                .execute(&mut *transaction)
                .await?;
        }

        if let Some(h) = root.base_hp_masters.first() {
            sqlx::query("INSERT INTO song_base_hp_master (hp) VALUES (?)")
                .bind(h.hp)
                .execute(&mut *transaction)
                .await?;
        }

        transaction.commit().await?;
        Ok(())
    }
}

fn count(row: &sqlx::mysql::MySqlRow, column: &str) -> usize {
    row.get::<i64, _>(column) as usize
}

async fn clear_master_tables(transaction: &mut Transaction<'_, MySql>) -> Result<(), AppError> {
    const TABLES: &[&str] = &[
        "master_version",
        "title_master",
        "song_select_master",
        "song_master",
        "song_score_rate_master",
        "song_base_score_master",
        "song_judge_zone_master",
        "song_base_hp_master",
        "song_hp_rate_master",
        "sound_sheet_name_master",
        "result_master",
    ];

    for table in TABLES {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&mut **transaction)
            .await?;
    }
    Ok(())
}

async fn bulk_insert<'a, T, F>(
    transaction: &mut Transaction<'_, MySql>,
    prefix: &str,
    items: &'a [T],
    mut binder: F,
    suffix: &str,
) -> Result<(), AppError>
where
    T: 'a,
    F: FnMut(sqlx::query_builder::Separated<'_, 'a, MySql, &str>, &'a T),
{
    if items.is_empty() {
        return Ok(());
    }

    let mut qb = QueryBuilder::new(prefix);
    qb.push_values(items, |separated, item| {
        binder(separated, item);
    });

    if !suffix.is_empty() {
        qb.push(" ");
        qb.push(suffix);
    }

    qb.build().execute(&mut **transaction).await?;
    Ok(())
}
