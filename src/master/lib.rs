use crate::master::{MasterDataResponse};

pub async fn sync_masters_all(pool: &sqlx::MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
let path = std::env::var("MASTER_DATA_PATH").unwrap_or_else(|_| "master/data.json".to_string());
    let data = std::fs::read_to_string(&path)?;
    let root: MasterDataResponse = serde_json::from_str(&data)?;
    for t in root.titles {
        sqlx::query("INSERT INTO title_master (tid) VALUES (?) ON DUPLICATE KEY UPDATE tid=tid")
            .bind(t.tid).execute(pool).await?;
    }
    for s in root.song_selects {
        sqlx::query("INSERT INTO song_select_master (`group`, start_song_time, song_time) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE start_song_time=VALUES(start_song_time), song_time=VALUES(song_time)")
            .bind(s.group).bind(s.start_song_time).bind(s.song_time).execute(pool).await?;
    }
    for s in root.songs {
        sqlx::query(r#"
            INSERT INTO song_master (sid, `group`, difficulty, name, composer, start_offset, bg)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE `group`=VALUES(`group`), difficulty=VALUES(difficulty), name=VALUES(name), composer=VALUES(composer), start_offset=VALUES(start_offset), bg=VALUES(bg)
        "#)
        .bind(s.sid).bind(s.group).bind(s.difficulty).bind(&s.name).bind(&s.composer).bind(s.start_offset).bind(s.bg)
        .execute(pool).await?;
    }
    for r in root.score_rates {
        sqlx::query("INSERT INTO song_score_rate_master (r_type, rate) VALUES (?, ?) ON DUPLICATE KEY UPDATE rate=VALUES(rate)")
            .bind(r.r_type).bind(r.rate).execute(pool).await?;
    }
    for j in root.judge_zones {
        sqlx::query("INSERT INTO song_judge_zone_master (j_type, zone) VALUES (?, ?) ON DUPLICATE KEY UPDATE zone=VALUES(zone)")
            .bind(j.j_type).bind(j.zone).execute(pool).await?;
    }
    for h in root.hp_rates {
        sqlx::query("INSERT INTO song_hp_rate_master (j_type, rate) VALUES (?, ?) ON DUPLICATE KEY UPDATE rate=VALUES(rate)")
            .bind(h.j_type).bind(h.rate).execute(pool).await?;
    }
    for s in root.sound_sheets {
        sqlx::query("INSERT INTO sound_sheet_name_master (category, id, sheet_name, cue_name) VALUES (?, ?, ?, ?) ON DUPLICATE KEY UPDATE sheet_name=VALUES(sheet_name), cue_name=VALUES(cue_name)")
            .bind(s.category).bind(s.id).bind(&s.sheet_name).bind(&s.cue_name).execute(pool).await?;
    }
    for r in root.results {
        sqlx::query("INSERT INTO result_master (rid) VALUES (?) ON DUPLICATE KEY UPDATE rid=rid")
            .bind(r.rid).execute(pool).await?;
    }
    if let Some(s) = root.base_scores.first() {
        sqlx::query("TRUNCATE TABLE song_base_score_master").execute(pool).await?;
        sqlx::query("INSERT INTO song_base_score_master (score) VALUES (?)").bind(s.score).execute(pool).await?;
    }
    if let Some(h) = root.base_hps.first() {
        sqlx::query("TRUNCATE TABLE song_base_hp_master").execute(pool).await?;
        sqlx::query("INSERT INTO song_base_hp_master (hp) VALUES (?)").bind(h.hp).execute(pool).await?;
    }
    println!("Master data synced successfully.");
    Ok(())
}
