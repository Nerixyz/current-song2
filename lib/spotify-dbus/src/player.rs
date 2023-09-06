use chrono::{DateTime, Utc};

use crate::interface::PlaybackStatus;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct State {
    pub timeline: Timeline,
    pub artist: String,
    pub title: String,
    pub cover_art: Option<String>,
    pub album: String,
    pub status: PlaybackStatus,
    pub playback_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Timeline {
    pub ts: DateTime<Utc>,
    pub duration: u64,
    pub position: i64,
}
