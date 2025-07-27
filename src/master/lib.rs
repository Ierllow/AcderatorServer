use crate::master::{MasterDataResponse};
use crate::common::bulk_insert;

pub async fn sync_masters_all(pool: &sqlx::MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::var("MASTER_DATA_PATH").expect("MASTER_DATA_PATH must be set");
    let data = std::fs::read_to_string(&path)?;
    let root: MasterDataResponse = serde_json::from_str(&data)?;
    let mut transaction = pool.begin().await?;
    sqlx::query("TRUNCATE TABLE master_version").execute(&mut *transaction).await?;
    sqlx::query("INSERT INTO master_version (version) VALUES (?)")
        .bind(&root.version)
        .execute(&mut *transaction)
        .await?;
    bulk_insert(
        &mut transaction,
        "INSERT INTO title_master (tid)",
        &root.titles,
        |mut b, t| { b.push_bind(t.tid); },
        "ON DUPLICATE KEY UPDATE tid=tid"
    ).await?;
    bulk_insert(
        &mut transaction,
        "INSERT INTO song_select_master (`group`, start_song_time, song_time)",
        &root.song_selects,
        |mut b, s| {
            b.push_bind(s.group)
            .push_bind(s.start_song_time)
            .push_bind(s.song_time);
        },
        "ON DUPLICATE KEY UPDATE start_song_time=VALUES(start_song_time), song_time=VALUES(song_time)"
    ).await?;
    bulk_insert(
        &mut transaction,
        "INSERT INTO song_master (sid, `group`, difficulty, name, composer, start_offset, bg)",
        &root.songs,
        |mut b, s| {
            b.push_bind(s.sid)
            .push_bind(s.group)
            .push_bind(s.difficulty)
            .push_bind(&s.name)
            .push_bind(&s.composer)
            .push_bind(s.start_offset)
            .push_bind(s.bg);
        },
        "ON DUPLICATE KEY UPDATE `group`=VALUES(`group`), difficulty=VALUES(difficulty), name=VALUES(name), composer=VALUES(composer), start_offset=VALUES(start_offset), bg=VALUES(bg)"
    ).await?;
    bulk_insert(
        &mut transaction,
        "INSERT INTO song_score_rate_master (r_type, rate)", 
        &root.score_rates,
        |mut b, r| { b.push_bind(r.r_type).push_bind(r.rate); },
        "ON DUPLICATE KEY UPDATE rate=VALUES(rate)"
    ).await?;
    bulk_insert(
        &mut transaction,
        "INSERT INTO song_judge_zone_master (j_type, zone)", 
        &root.judge_zones,
        |mut b, j| { b.push_bind(j.j_type).push_bind(j.zone); },
        "ON DUPLICATE KEY UPDATE zone=VALUES(zone)"
    ).await?;
    bulk_insert(
        &mut transaction,
        "INSERT INTO song_hp_rate_master (j_type, rate)", 
        &root.hp_rates,
        |mut b, h| { b.push_bind(h.j_type).push_bind(h.rate); },
        "ON DUPLICATE KEY UPDATE rate=VALUES(rate)"
    ).await?;
    bulk_insert(
        &mut transaction,
        "INSERT INTO sound_sheet_name_master (category, id, sheet_name, cue_name)",
        &root.sound_sheets,
        |mut b, s| {
            b.push_bind(s.category)
            .push_bind(s.id)
            .push_bind(&s.sheet_name)
            .push_bind(&s.cue_name);
        },
        "ON DUPLICATE KEY UPDATE sheet_name=VALUES(sheet_name), cue_name=VALUES(cue_name)"
    ).await?;
    bulk_insert(
        &mut transaction,
        "INSERT INTO result_master (rid)",
        &root.results,
        |mut b, r| { b.push_bind(r.rid); },
        "ON DUPLICATE KEY UPDATE rid=rid"
    ).await?;
    if let Some(s) = root.base_scores.first() {
        sqlx::query("TRUNCATE TABLE song_base_score_master").execute(&mut *transaction).await?;
        sqlx::query("INSERT INTO song_base_score_master (score) VALUES (?)")
            .bind(s.score)
            .execute(&mut *transaction)
            .await?;
    }
    if let Some(h) = root.base_hps.first() {
        sqlx::query("TRUNCATE TABLE song_base_hp_master").execute(&mut *transaction).await?;
        sqlx::query("INSERT INTO song_base_hp_master (hp) VALUES (?)")
            .bind(h.hp)
            .execute(&mut *transaction)
            .await?;
    }

    transaction.commit().await?;

    println!("MasterData synced successfully.");
    Ok(())
}
