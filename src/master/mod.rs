use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt::Debug;

pub mod lib;

#[derive(Debug, Serialize, Deserialize)]
pub struct MasterDataResponse {
    pub version_master: String,
    pub title_masters: Vec<TitleMaster>,
    pub song_select_masters: Vec<SongSelectMaster>,
    pub song_masters: Vec<SongMaster>,
    pub score_rate_masters: Vec<SongScoreRateMaster>,
    pub base_score_masters: Vec<SongBaseScoreMaster>,
    pub judge_zone_masters: Vec<SongJudgeZoneMaster>,
    pub base_hp_masters: Vec<SongBaseHpMaster>,
    pub hp_rate_masters: Vec<SongHpRateMaster>,
    pub sound_sheet_masters: Vec<SoundSheetNameMaster>,
    pub result_masters: Vec<ResultMaster>,
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
