use crate::{
    model::{
        AlbumModel, AutoRepeatMode, MediaModel, PlaybackModel, PlaybackStatus, PlaybackType,
        TimelineModel,
    },
    util::ResultExt,
};
use std::convert::{TryFrom, TryInto};
use windows::{
    core::HRESULT,
    Media::{
        Control::{
            GlobalSystemMediaTransportControlsSessionMediaProperties,
            GlobalSystemMediaTransportControlsSessionPlaybackInfo,
            GlobalSystemMediaTransportControlsSessionPlaybackStatus,
            GlobalSystemMediaTransportControlsSessionTimelineProperties,
        },
        MediaPlaybackAutoRepeatMode, MediaPlaybackType,
    },
};

impl TryFrom<GlobalSystemMediaTransportControlsSessionPlaybackInfo> for PlaybackModel {
    type Error = windows::core::Error;

    fn try_from(
        value: GlobalSystemMediaTransportControlsSessionPlaybackInfo,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            status: value.PlaybackStatus()?.try_into()?,
            r#type: value
                .PlaybackType()?
                .Value()
                .map(From::from)
                .unwrap_or_default(),
            auto_repeat: value
                .AutoRepeatMode()
                .and_then(|r| r.Value())
                .map(From::from)
                .unwrap_or_default(),
            rate: value.PlaybackRate().and_then(|r| r.Value()).unwrap_or(1.0),
            shuffle: value
                .IsShuffleActive()
                .and_then(|r| r.Value())
                .unwrap_or_default(),
        })
    }
}

impl TryFrom<GlobalSystemMediaTransportControlsSessionTimelineProperties> for TimelineModel {
    type Error = windows::core::Error;

    fn try_from(
        value: GlobalSystemMediaTransportControlsSessionTimelineProperties,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            start: value.StartTime()?.Duration,
            end: value.EndTime()?.Duration,
            position: value.Position()?.Duration,
            last_updated_at_ms: filetime_to_unix_ms(value.LastUpdatedTime()?.UniversalTime),
        })
    }
}

impl TryFrom<GlobalSystemMediaTransportControlsSessionMediaProperties> for MediaModel {
    type Error = windows::core::Error;

    fn try_from(
        value: GlobalSystemMediaTransportControlsSessionMediaProperties,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            title: value.Title()?.to_string(),
            subtitle: value.Subtitle()?.to_string(),
            artist: value.Artist()?.to_string(),
            track_number: value
                .TrackNumber()
                .opt()?
                .and_then(|v: i32| v.try_into().ok()),
            album: (&value).try_into().ok(),
            genres: value.Genres()?.into_iter().map(|s| s.to_string()).collect(),
            playback_type: value
                .PlaybackType()?
                .Value()
                .map(Into::into)
                .unwrap_or_default(),
        })
    }
}

impl TryFrom<&GlobalSystemMediaTransportControlsSessionMediaProperties> for AlbumModel {
    type Error = ();

    fn try_from(
        value: &GlobalSystemMediaTransportControlsSessionMediaProperties,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            artist: value
                .AlbumArtist()
                .opt()
                .map_err(|_| ())?
                .ok_or(())?
                .to_string(),
            title: value
                .AlbumTitle()
                .opt()
                .map_err(|_| ())?
                .ok_or(())?
                .to_string(),
            track_count: value
                .AlbumTrackCount()
                .opt()
                .map_err(|_| ())?
                .and_then(|v: i32| v.try_into().ok())
                .ok_or(())?,
        })
    }
}

impl TryFrom<GlobalSystemMediaTransportControlsSessionPlaybackStatus> for PlaybackStatus {
    type Error = windows::core::Error;

    fn try_from(
        value: GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    ) -> Result<Self, Self::Error> {
        match value {
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed => Ok(Self::Closed),
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Opened => Ok(Self::Opened),
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Changing => Ok(Self::Changing),
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped => Ok(Self::Stopped),
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => Ok(Self::Playing),
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => Ok(Self::Paused),
            _ => Err(HRESULT(0x1fe2).into()),
        }
    }
}

impl From<MediaPlaybackType> for PlaybackType {
    fn from(value: MediaPlaybackType) -> Self {
        match value {
            MediaPlaybackType::Unknown => Self::Unknown,
            MediaPlaybackType::Image => Self::Image,
            MediaPlaybackType::Music => Self::Music,
            MediaPlaybackType::Video => Self::Video,
            _ => Self::Unknown,
        }
    }
}

impl From<MediaPlaybackAutoRepeatMode> for AutoRepeatMode {
    fn from(value: MediaPlaybackAutoRepeatMode) -> Self {
        match value {
            MediaPlaybackAutoRepeatMode::None => Self::None,
            MediaPlaybackAutoRepeatMode::List => Self::List,
            MediaPlaybackAutoRepeatMode::Track => Self::Track,
            _ => Self::None,
        }
    }
}

fn filetime_to_unix_ms(filetime: i64) -> i64 {
    (filetime / 10000).checked_sub(11644473600000).unwrap_or(0)
}
