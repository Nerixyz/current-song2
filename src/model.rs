use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum ModuleState {
    Playing(PlayInfo),
    Paused,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlayInfo {
    pub title: String,
    pub artist: String,
    pub track_number: Option<u32>,

    pub image: Option<ImageInfo>,
    pub timeline: Option<TimelineInfo>,
    pub album: Option<AlbumInfo>,

    pub source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum ImageInfo {
    External(String),
    Internal(InternalImage),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InternalImage {
    pub id: usize,
    pub epoch_id: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimelineInfo {
    pub ts: u64,

    pub duration_ms: u64,
    pub progress_ms: u64,

    pub rate: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AlbumInfo {
    pub title: String,
    pub track_count: u32,
}

impl PlayInfo {
    #[cfg(test)]
    pub fn simple<Title, Artist, Source>(title: Title, artist: Artist, source: Source) -> Self
    where
        Title: Into<String>,
        Artist: Into<String>,
        Source: Into<String>,
    {
        Self {
            title: title.into(),
            artist: artist.into(),
            track_number: None,
            image: None,
            timeline: None,
            album: None,
            source: source.into(),
        }
    }
}
