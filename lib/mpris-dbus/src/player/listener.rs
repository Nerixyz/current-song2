use std::collections::HashMap;

use crate::interface::*;
use crate::player::State;
use chrono::Utc;
use futures::StreamExt;
use tap::TapFallible;
use tokio::sync::mpsc;
use tracing::info;
use tracing::warn;
use zbus::{zvariant, Connection};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Failed to get DBus connection to the user message bus")]
    GetConnection(zbus::Error),
    #[error("Failed to setup the proxy")]
    SetupProxy(zbus::Error),
}

macro_rules! send_or_break {
    ($tx:ident, $it:expr) => {
        if let Err(_) = $tx.send($it).await {
            break;
        }
    };
}

pub async fn listen<D>(dest: D) -> Result<mpsc::Receiver<State>, Error>
where
    D: TryInto<zbus::names::BusName<'static>>,
    D::Error: Into<zbus::Error>,
{
    let connection = Connection::session().await.map_err(Error::GetConnection)?;
    let proxy = SpotifyPlayerProxy::builder(&connection)
        .destination(dest)
        .map_err(Error::SetupProxy)?
        .build()
        .await
        .map_err(Error::SetupProxy)?;

    let (tx, rx) = mpsc::channel(8);

    tokio::spawn(async move {
        let mut status_changed = proxy.receive_playback_status_changed().await;
        let mut meta_changed = proxy.receive_metadata_changed().await;
        let mut rate_changed = proxy.receive_rate_changed().await;
        let mut seeked = proxy.receive_seeked().await.expect("TODO");
        let mut owner_changed = proxy.receive_owner_changed().await.expect("TODO");

        let mut state = State::default();

        loop {
            tokio::select! { biased;
                Some(status) = status_changed.next() => {
                    match status.get().await {
                        Ok(s) => {
                            state.status = s;
                            send_or_break!(tx, state.clone())
                        },
                        Err(e) => warn!(error = %e, "Failed to get status"),
                    };
                },
                Some(_) = meta_changed.next() => {
                    update_meta(&proxy, &mut state).await;
                    send_or_break!(tx, state.clone());
                },
                Some(rate) = rate_changed.next() => {
                    match rate.get().await {
                        Ok(r) => {
                            state.playback_rate = r;
                            send_or_break!(tx, state.clone())
                        },
                        Err(e) => warn!(error = %e, "Failed to get rate"),
                    };
                },
                Some(_) = seeked.next() => {
                    if update_position(&proxy, &mut state).await {
                        send_or_break!(tx, state.clone());
                    }
                },
                Some(owner) = owner_changed.next() => {
                    if owner.filter(|x| x.is_empty()).is_none() {
                        state.status = PlaybackStatus::Stopped;
                        send_or_break!(tx, state.clone());
                    }
                },
                else => break
            }
        }
        info!("loop ended");
    });

    Ok(rx)
}

async fn update_meta(proxy: &SpotifyPlayerProxy<'_>, state: &mut State) {
    let Ok(meta) = proxy
        .metadata()
        .await
        .tap_err(|e| warn!(error = %e, "Failed to get metadata"))
    else {
        return;
    };

    get_meta(&meta, "mpris:length", &mut state.timeline.duration, Some);
    get_meta(&meta, "xseam:title", &mut state.title, Some);
    get_meta(&meta, "xseam:album", &mut state.album, Some);
    get_meta(
        &meta,
        "xseam:artist",
        &mut state.artist,
        |artist: zvariant::Array| {
            artist
                .iter()
                .filter_map(|v| zvariant::Str::try_from(v).ok())
                .fold(String::new(), |acc, v| {
                    if acc.is_empty() {
                        v.to_string()
                    } else {
                        format!("{acc}, {v}")
                    }
                })
        },
    );
    get_meta(&meta, "mpris:artUrl", &mut state.cover_art, Some);

    update_position(proxy, state).await;
}

async fn update_position(proxy: &SpotifyPlayerProxy<'_>, state: &mut State) -> bool {
    match proxy.position().await {
        Ok(p) => {
            state.timeline.position = p;
            state.timeline.ts = Utc::now();
            true
        }
        Err(e) => {
            warn!(error = %e, "Failed to get rate");
            false
        }
    }
}

fn get_meta<'a, T, U>(
    meta: &'a HashMap<zvariant::Str<'_>, zvariant::Value<'a>>,
    key: &'static str,
    target: &mut U,
    map: impl FnOnce(T) -> U,
) where
    T: TryFrom<&'a zvariant::Value<'a>>,
    U: Default,
{
    let value = meta
        .get(&zvariant::Str::from_static(key))
        .and_then(|v| T::try_from(v).ok());
    match value {
        Some(v) => *target = map(v),
        None => *target = U::default(),
    }
}
