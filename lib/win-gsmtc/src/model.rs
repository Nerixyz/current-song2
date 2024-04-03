use std::{
    fmt::{Debug, Formatter},
    time::Duration,
};

/// Represents a playback session from another app providing info about that session.
///
/// This model represents a [`GlobalSystemMediaTransportControlsSession`](https://learn.microsoft.com/uwp/api/windows.media.control.globalsystemmediatransportcontrolssession).
/// It's gradually updated through the [`SessionUpdateEvent`](crate::SessionUpdateEvent).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionModel {
    /// The current playback info for this session.
    pub playback: Option<PlaybackModel>,
    /// The object representing the timeline property values.
    pub timeline: Option<TimelineModel>,
    /// The current media item.
    ///
    /// This doesn't include the thumbnail. The thumbnail is included in [`SessionUpdateEvent::Media`](crate::SessionUpdateEvent::Media).
    pub media: Option<MediaModel>,
    /// The app user model id.
    ///
    /// This corresponds to the [`SourceAppUserModelId`](https://learn.microsoft.com/uwp/api/windows.media.control.globalsystemmediatransportcontrolssession.sourceappusermodelid).
    pub source: String,
}

/// An image read from a [`IRandomAccessStreamWithContentType`](https://learn.microsoft.com/uwp/api/windows.storage.streams.irandomaccessstreamwithcontenttype).
#[derive(Eq, PartialEq)]
pub struct Image {
    /// The identifier for the format of the data.
    pub content_type: String,
    /// The raw bytes of the data.
    pub data: Vec<u8>,
}

impl Debug for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("content_type", &self.content_type)
            .finish()
    }
}

/// The object that holds all of the playback information about a session (Play state, playback type etc.).
///
/// Controls are not yet implemented.
///
/// This model represents a [`GlobalSystemMediaTransportControlsSessionPlaybackInfo`](https://learn.microsoft.com/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessionplaybackinfo).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// TODO: controls
pub struct PlaybackModel {
    /// The current playback state of the session.
    pub status: PlaybackStatus,
    /// Specifies what type of content the session has.
    pub r#type: PlaybackType,
    /// The rate at which playback is happening.
    pub rate: f64,
    /// Specifies whether the session is currently playing content in a shuffled order or not.
    pub shuffle: bool,
    /// Specifies the repeat mode of the session.
    pub auto_repeat: AutoRepeatMode,
}

/// Represents the timeline state of the session (Position, seek ranges etc.).
///
/// This model represents a [`GlobalSystemMediaTransportControlsSessionTimelineProperties`](https://learn.microsoft.com/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessiontimelineproperties).
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimelineModel {
    /// The starting timestamp of the current media item.
    pub start: Duration,
    /// The end timestamp of the current media item.
    pub end: Duration,
    /// The playback position, current as of [`last_updated_at_ms`](TimelineModel::last_updated_at_ms).
    pub position: Duration,
    /// The UTC time at which the timeline properties were last updated.
    pub last_updated_at_ms: i64,
    // TODO: add {Max,Min}SeekTime
}

/// Holds information about the content that the current session has.
///
/// This model represents a [`GlobalSystemMediaTransportControlsSessionMediaProperties`](https://learn.microsoft.com/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessionmediaproperties).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MediaModel {
    /// The title.
    pub title: String,
    /// The subtitle of the media.
    pub subtitle: String,
    /// The Artist name.
    pub artist: String,

    /// Information about the album this media is contained in.
    pub album: Option<AlbumModel>,
    /// The number associated with this track.
    pub track_number: Option<u32>,

    /// A list of all strings representing the genres.
    pub genres: Vec<String>,
    /// The playback type of the content.
    pub playback_type: PlaybackType,
}

/// Holds information about an album.
///
/// This doesn't represent a WinRT class - it's a convenience wrapper.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AlbumModel {
    /// The name of the album artist.
    pub artist: String,
    /// The title of the album.
    pub title: String,
    /// The total number of tracks on the album.
    pub track_count: u32,
}

/// The different states of playback the session could be in.
///
/// This represents a [`GlobalSystemMediaTransportControlsSessionPlaybackStatus`](https://learn.microsoft.com/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessionplaybackstatus).
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlaybackStatus {
    /// The media is closed.
    Closed,
    /// The media is opened.
    Opened,
    /// The media is changing.
    Changing,
    /// The media is stopped.
    Stopped,
    /// The media is playing.
    Playing,
    /// The media is paused.
    Paused,
}

/// Defines values for the types of media playback.
///
/// This represents a [`MediaPlaybackType`](https://learn.microsoft.com/uwp/api/windows.media.mediaplaybacktype).
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlaybackType {
    /// The media type is unknown.
    Unknown,
    /// The media type is audio music.
    Music,
    /// The media type is video.
    Video,
    /// The media type is an image.
    Image,
}

impl Default for PlaybackType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Specifies the auto repeat mode for media playback.
///
/// This represents a [`MediaPlaybackAutoRepeatMode`](https://learn.microsoft.com/uwp/api/windows.media.mediaplaybackautorepeatmode).
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AutoRepeatMode {
    /// No repeating.
    None,
    /// Repeat the current track.
    Track,
    /// Repeat the current list of tracks.
    List,
}

impl Default for AutoRepeatMode {
    fn default() -> Self {
        Self::None
    }
}
