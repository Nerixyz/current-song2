use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use zbus::{fdo, proxy, zvariant};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, zvariant::Type, Deserialize, Serialize)]
#[zvariant(signature = "s")]
pub enum PlaybackStatus {
    Playing,
    Paused,
    #[default]
    Stopped,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, zvariant::Type, Deserialize, Serialize)]
#[zvariant(signature = "s")]
pub enum LoopStatus {
    #[default]
    None,
    Track,
    Playlist,
}

#[proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_path = "/org/mpris/MediaPlayer2",
    gen_blocking = false
)]
pub trait MediaPlayer {
    // Methods
    fn next(&self) -> fdo::Result<()>;
    fn previous(&self) -> fdo::Result<()>;
    fn pause(&self) -> fdo::Result<()>;
    fn play_pause(&self) -> fdo::Result<()>;
    fn stop(&self) -> fdo::Result<()>;
    fn play(&self) -> fdo::Result<()>;
    fn seek(&self, offset: i64) -> fdo::Result<()>;
    fn set_position(&self, track_id: zvariant::ObjectPath<'_>, position: i64) -> fdo::Result<()>;
    fn open_uri(&self, uri: zvariant::Str<'_>) -> fdo::Result<()>;

    // Signals
    #[zbus(signal)]
    fn seeked(&self, position: i64) -> fdo::Result<()>;

    // Properties
    #[zbus(property(emits_changed_signal = "true"))]
    fn playback_status(&self) -> fdo::Result<PlaybackStatus>;

    #[zbus(property(emits_changed_signal = "true"))]
    fn loop_status(&self) -> fdo::Result<LoopStatus>;
    #[zbus(property(emits_changed_signal = "true"))]
    fn set_loop_status(&self, status: LoopStatus) -> fdo::Result<()>;

    #[zbus(property(emits_changed_signal = "true"))]
    fn rate(&self) -> fdo::Result<f64>;
    #[zbus(property(emits_changed_signal = "true"))]
    fn set_rate(&self, rate: f64) -> fdo::Result<()>;

    #[zbus(property(emits_changed_signal = "true"))]
    fn shuffle(&self) -> fdo::Result<bool>;
    #[zbus(property(emits_changed_signal = "true"))]
    fn set_shuffle(&self, enabled: bool) -> fdo::Result<()>;

    #[zbus(property(emits_changed_signal = "true"))]
    fn volume(&self) -> fdo::Result<f64>;
    #[zbus(property(emits_changed_signal = "true"))]
    fn set_volume(&self, volume: f64) -> fdo::Result<()>;

    #[zbus(property(emits_changed_signal = "true"))]
    fn metadata(&self) -> fdo::Result<HashMap<zvariant::Str<'static>, zvariant::Value>>;

    #[zbus(property(emits_changed_signal = "false"))]
    fn position(&self) -> fdo::Result<i64>;
    #[zbus(property)]
    fn minimum_rate(&self) -> fdo::Result<i64>;
    #[zbus(property)]
    fn maximum_rate(&self) -> fdo::Result<f64>;
    #[zbus(property)]
    fn can_go_next(&self) -> fdo::Result<bool>;
    #[zbus(property)]
    fn can_go_previous(&self) -> fdo::Result<bool>;
    #[zbus(property)]
    fn can_play(&self) -> fdo::Result<bool>;
    #[zbus(property)]
    fn can_pause(&self) -> fdo::Result<bool>;
    #[zbus(property)]
    fn can_seek(&self) -> fdo::Result<bool>;
    #[zbus(property)]
    fn can_control(&self) -> fdo::Result<bool>;
}

macro_rules! string_enum_conversions {
    ($en:ty; $($val:ident => $str:literal),+) => {
        impl<'a> From<$en> for zvariant::Value<'a> {
            fn from(value: $en) -> Self {
                zvariant::Value::Str(zvariant::Str::from_static(match value {
                    $(<$en>::$val => $str),+
                }))
            }
        }

        impl TryFrom<zvariant::OwnedValue> for $en {
            type Error = zvariant::Error;

            fn try_from(value: zvariant::OwnedValue) -> Result<Self, Self::Error> {
                let Ok(s) = <&str>::try_from(&value) else {
                    return Err(Self::Error::IncorrectType);
                };
                match s {
                    $($str => Ok(Self::$val)),+,
                    _ => Err(Self::Error::IncorrectType),
                }
            }
        }
    };
}

string_enum_conversions! { LoopStatus;
    None => "None",
    Track => "Track",
    Playlist => "Playlist"
}

string_enum_conversions! { PlaybackStatus;
    Playing => "Playing",
    Paused => "Paused",
    Stopped => "Stopped"
}
