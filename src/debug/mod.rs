use crate::query::song::SongRecord;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod api;
pub mod lib;

#[derive(Serialize)]
struct MasterDebugData {
    version: String,
    base_score: Option<i32>,
    counts: MasterCounts,
    raw: Value,
}

#[derive(Serialize)]
struct DebugSong {
    sid: i32,
    group: i32,
    difficulty: i32,
    name: String,
    composer: String,
    start_offset: f32,
    bg: i32,
}

impl From<SongRecord> for DebugSong {
    fn from(song: SongRecord) -> Self {
        Self {
            sid: song.sid,
            group: song.group,
            difficulty: song.difficulty,
            name: song.name,
            composer: song.composer,
            start_offset: song.start_offset,
            bg: song.bg,
        }
    }
}

#[derive(Deserialize)]
struct SaveMasterVersionRequest {
    version: String,
}

#[derive(Deserialize)]
struct SaveBaseScoreRequest {
    score: i32,
}

#[derive(Deserialize)]
struct SaveSongRequest {
    sid: i32,
    group: i32,
    difficulty: i32,
    name: String,
    composer: String,
    start_offset: f32,
    bg: i32,
}

#[derive(Serialize)]
struct MasterSaveResponse {
    ok: bool,
    message: &'static str,
}

#[derive(Serialize)]
struct MasterCounts {
    master_version: usize,
    title_masters: usize,
    song_select_masters: usize,
    song_masters: usize,
    score_rate_masters: usize,
    base_score_masters: usize,
    judge_zone_masters: usize,
    base_hp_masters: usize,
    hp_rate_masters: usize,
    sound_sheet_masters: usize,
    result_masters: usize,
}
