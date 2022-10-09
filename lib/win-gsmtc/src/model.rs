use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone)]
pub struct SessionModel {
    pub playback: Option<PlaybackModel>,
    pub timeline: Option<TimelineModel>,
    pub media: Option<MediaModel>,
    pub source: String,
}

pub struct Image {
    pub content_type: String,
    pub data: Vec<u8>,
}

impl Debug for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("content_type", &self.content_type)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackModel {
    // TODO: controls
    pub status: PlaybackStatus,
    pub r#type: PlaybackType,
    pub rate: f64,
    pub shuffle: bool,
    pub auto_repeat: AutoRepeatMode,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TimelineModel {
    pub start: i64,
    pub end: i64,
    pub position: i64,
    pub last_updated_at_ms: i64,
}

#[derive(Debug, Clone)]
pub struct MediaModel {
    pub title: String,
    pub subtitle: String,
    pub artist: String,

    pub album: Option<AlbumModel>,
    pub track_number: Option<u32>,

    pub genres: Vec<String>,
    pub playback_type: PlaybackType,
}

#[derive(Debug, Clone)]
pub struct AlbumModel {
    pub artist: String,
    pub title: String,
    pub track_count: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PlaybackStatus {
    Closed,
    Opened,
    Changing,
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PlaybackType {
    Unknown,
    Music,
    Video,
    Image,
}

impl Default for PlaybackType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AutoRepeatMode {
    None,
    Track,
    List,
}

impl Default for AutoRepeatMode {
    fn default() -> Self {
        Self::None
    }
}
