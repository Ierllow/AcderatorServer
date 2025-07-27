use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt::Debug;

pub mod lib;

#[derive(Debug, Serialize, Deserialize)]
pub struct MasterDataResponse {
    pub titles: Vec<TitleMaster>,
    pub song_selects: Vec<SongSelectMaster>,
    pub songs: Vec<SongMaster>,
    pub score_rates: Vec<SongScoreRateMaster>,
    pub base_scores: Vec<SongBaseScoreMaster>,
    pub judge_zones: Vec<SongJudgeZoneMaster>,
    pub base_hps: Vec<SongBaseHpMaster>,
    pub hp_rates: Vec<SongHpRateMaster>,
    pub sound_sheets: Vec<SoundSheetNameMaster>,
    pub results: Vec<ResultMaster>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TitleMaster {
    pub tid: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SongSelectMaster {
    pub group: i32,
    pub start_song_time: i32,
    pub song_time: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SongMaster {
    pub sid: i32,
    pub group: i32,
    pub difficulty: i32,
    pub name: String,
    pub composer: String,
    pub start_offset: f32,
    pub bg: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SongScoreRateMaster {
    pub r_type: i32,
    pub rate: f32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SongBaseScoreMaster {
    pub score: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SongJudgeZoneMaster {
    #[sqlx(rename = "j_type")]
    pub j_type: i32,
    pub zone: f32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SongBaseHpMaster {
    pub hp: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SongHpRateMaster {
    #[sqlx(rename = "j_type")]
    pub j_type: i32,
    pub rate: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SoundSheetNameMaster {
    pub category: i32,
    pub id: i32,
    pub sheet_name: String,
    pub cue_name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ResultMaster {
    pub rid: i32,
}
