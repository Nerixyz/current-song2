use crate::{
    actors::manager::{CreateModule, Manager, RemoveModule, UpdateModule},
    model::{AlbumInfo, ImageInfo, ModuleState, PlayInfo, TimelineInfo},
};
use actix::Addr;
use anyhow::Result as AnyResult;
use mpris_dbus::{interface::PlaybackStatus, player};
use tap::TapFallible;
use tokio::sync::mpsc;
use tracing::{span, warn, Instrument, Level};

#[derive(Debug)]
struct DBusWorker {
    manager: Addr<Manager>,
    module_id: usize,
    paused: bool,
    source: String,
}

pub async fn start_spawning(manager: Addr<Manager>, sources: &[String]) -> AnyResult<()> {
    for name in sources {
        let source = name.clone();
        let manager = manager.clone();
        let Ok(module_id) = manager
            .send(CreateModule { priority: 0 })
            .await
            .tap_err(|e| warn!(error = %e, "Failed to create module"))
        else {
            continue;
        };
        tokio::spawn(
            async move {
                let worker = DBusWorker {
                    manager,
                    module_id,
                    paused: false,
                    source,
                };
                let Ok(rx) = player::listen(worker.source.clone())
                    .await
                    .tap_err(|e| warn!(error = %e, "Failed to listen"))
                else {
                    return;
                };
                worker.feed_manager(rx).await;
            }
            .instrument(span!(Level::INFO, "DBusWorker", source = name)),
        );
    }
    Ok(())
}

impl DBusWorker {
    async fn feed_manager(mut self, mut rx: mpsc::Receiver<player::State>) {
        while let Some(evt) = rx.recv().await {
            self.send_update(evt).await;
        }
        self.manager
            .send(RemoveModule { id: self.module_id })
            .await
            .ok();
    }

    async fn send_update(&mut self, state: player::State) {
        let paused = state.status != PlaybackStatus::Playing;
        if paused && self.paused {
            return;
        }
        let state = convert_model(state, &self.source);
        self.paused = paused;
        let span = span!(Level::TRACE, "Update Module", id = self.module_id, state = ?state, paused = self.paused);
        self.manager
            .send(UpdateModule {
                id: self.module_id,
                state,
            })
            .instrument(span)
            .await
            .ok();
    }
}

fn convert_model(from: player::State, source: &str) -> ModuleState {
    if from.status != PlaybackStatus::Playing {
        return ModuleState::Paused;
    }
    return ModuleState::Playing(PlayInfo {
        title: from.title.unwrap_or_default(),
        artist: from.artist,
        track_number: None,
        image: from.cover_art.map(ImageInfo::External),
        timeline: Some(TimelineInfo {
            ts: from
                .timeline
                .ts
                .timestamp_millis()
                .try_into()
                .unwrap_or_default(),
            duration_ms: from.timeline.duration.unwrap_or(0) / 1000,
            progress_ms: (from.timeline.position / 1000)
                .try_into()
                .unwrap_or_default(),
            rate: from.playback_rate as f32,
        }),
        album: from.album.map(|title| AlbumInfo {
            title,
            track_count: 0,
        }),
        source: format!("dbus::{source}"),
    });
}
