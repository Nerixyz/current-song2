use std::sync::Arc;

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
    #[error("Failed to listen to property-change events")]
    SetupStream(zbus::Error),
}

macro_rules! send_or_break {
    ($tx:ident, $it:expr) => {
        if let Err(_) = $tx.send($it).await {
            break;
        }
    };
}

macro_rules! get_meta {
    ($meta:ident, $key:literal, $typ:ty) => {
        $meta
            .get(&zvariant::Str::from_static($key))
            .and_then(|v| <$typ>::try_from(v).ok())
    };
}

pub async fn listen<D>(dest: D) -> Result<mpsc::Receiver<Arc<State>>, Error>
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
            tokio::select! {
                Some(status) = status_changed.next() => {
                    match status.get().await {
                        Ok(s) => {
                            state.status = s;
                            send_or_break!(tx, Arc::new(state.clone()))
                        },
                        Err(e) => warn!(error = %e, "Failed to get status"),
                    };
                },
                Some(_) = meta_changed.next() => {
                    update_meta(&proxy, &mut state).await;
                    send_or_break!(tx, Arc::new(state.clone()));
                },
                Some(rate) = rate_changed.next() => {
                    match rate.get().await {
                        Ok(r) => {
                            state.playback_rate = r;
                            send_or_break!(tx, Arc::new(state.clone()))
                        },
                        Err(e) => warn!(error = %e, "Failed to get rate"),
                    };
                },
                Some(_) = seeked.next() => {
                    match proxy.position().await {
                        Ok(p) => {
                            state.timeline.position = p;
                            state.timeline.ts = Utc::now();
                            send_or_break!(tx, Arc::new(state.clone()));
                        },
                        Err(e) => warn!(error = %e, "Failed to get rate"),
                    };
                },
                Some(owner) = owner_changed.next() => {
                    if owner.filter(|x| x.is_empty()).is_none() {
                        state.status = PlaybackStatus::Stopped;
                        send_or_break!(tx, Arc::new(state.clone()));
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

    if let Some(length) = get_meta!(meta, "mpris:length", u64) {
        state.timeline.duration = length;
    }
    if let Some(title) = get_meta!(meta, "xesam:title", String) {
        state.title = title;
    }
    if let Some(artist) = get_meta!(meta, "xesam:artist", zvariant::Array) {
        state.artist = artist
            .iter()
            .filter_map(|v| zvariant::Str::try_from(v).ok())
            .fold(String::new(), |acc, v| {
                if acc.is_empty() {
                    v.to_string()
                } else {
                    format!("{acc}, {v}")
                }
            });
    }
    if let Some(album) = get_meta!(meta, "xesam:album", String) {
        state.album = album;
    };
    state.cover_art = get_meta!(meta, "mpris:artUrl", String);

    let Ok(position) = proxy
        .position()
        .await
        .tap_err(|e| warn!(error = %e, "Failed to get position"))
    else {
        return;
    };
    state.timeline.position = position;
    state.timeline.ts = Utc::now();
}
