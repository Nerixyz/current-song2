use chrono::{DateTime, Utc};

use crate::interface::PlaybackStatus;

mod listener;

pub use listener::{listen, Error as ListenError};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct State {
    pub title: Option<String>,
    pub artist: String,
    pub track_number: Option<i32>,
    pub album: Option<String>,
    pub cover_art: Option<String>,

    pub status: PlaybackStatus,
    pub playback_rate: f64,
    pub timeline: Timeline,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Timeline {
    pub ts: DateTime<Utc>,
    pub duration: Option<u64>,
    pub position: i64,
}
