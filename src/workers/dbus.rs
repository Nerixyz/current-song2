use std::{ffi::OsStr, path::Path, sync::Arc};

use crate::{
    actors::manager::{CreateModule, Manager, RemoveModule, UpdateModule},
    config::CONFIG,
    image_store::{ImageStore, SlotRef},
    model::{AlbumInfo, ImageInfo, InternalImage, ModuleState, PlayInfo, TimelineInfo},
};
use actix::Addr;
use anyhow::Result as AnyResult;
use futures::StreamExt;
use mpris_dbus::{interface::PlaybackStatus, player};
use std::sync::RwLock;
use tap::TapFallible;
use tokio::sync::mpsc;
use tracing::{debug, info, span, warn, Instrument, Level};
use url::Url;

#[derive(Debug)]
struct DBusWorker {
    manager: Addr<Manager>,
    module_id: usize,
    paused: bool,
    source: zbus_names::BusName<'static>,
    image_store: Arc<RwLock<ImageStore>>,
    image_id: SlotRef,
}

pub async fn start_spawning(
    manager: Addr<Manager>,
    image_store: Arc<RwLock<ImageStore>>,
) -> AnyResult<()> {
    let discoverer = mpris_dbus::discovery::Listener::new().await?;
    let mut name_stream = discoverer.listen().await?;
    tokio::spawn(async move {
        while let Some(name) = name_stream.next().await {
            if !CONFIG
                .modules
                .dbus
                .destinations
                .iter()
                .any(|s| fast_glob::glob_match(s.as_bytes(), name.as_bytes()))
            {
                debug!("Ignoring {}", name);
                continue;
            }
            info!("Listening to dbus service {}", name);

            let manager = manager.clone();
            let Ok(module_id) = manager
                .send(CreateModule { priority: 0 })
                .await
                .tap_err(|e| warn!(error = %e, "Failed to create module"))
            else {
                continue;
            };
            let image_store = image_store.clone();
            let image_id = SlotRef::new(&image_store);
            let source = name.clone();
            tokio::spawn(
                async move {
                    let worker = DBusWorker {
                        manager,
                        module_id,
                        paused: false,
                        source,
                        image_store,
                        image_id,
                    };
                    let Ok(rx) = player::listen(worker.source.clone())
                        .await
                        .tap_err(|e| warn!(error = %e, "Failed to listen"))
                    else {
                        return;
                    };
                    worker.feed_manager(rx).await;
                }
                .instrument(span!(Level::INFO, "DBusWorker", source = %name)),
            );
        }
    });

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
        let state = self.convert_model(state).await;
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

    async fn convert_model(&mut self, from: player::State) -> ModuleState {
        if from.status != PlaybackStatus::Playing {
            return ModuleState::Paused;
        }

        let image = match from.cover_art {
            Some(url) => self.make_image(url).await,
            None => None,
        };
        if image.is_none() {
            self.image_store.write().unwrap().clear(*self.image_id);
        }

        return ModuleState::Playing(PlayInfo {
            title: from.title.unwrap_or_default(),
            artist: from.artist,
            track_number: from.track_number.map(|n| n as u32),
            image,
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
            source: format!("dbus::{}", self.source),
        });
    }

    async fn make_image(&mut self, url_string: String) -> Option<ImageInfo> {
        let url = Url::parse(&url_string)
            .inspect_err(
                |e| warn!(url = url_string, error=%e, "Failed to parse image url from dbus"),
            )
            .ok()?;

        if url.scheme() == "file" {
            let bytes = tokio::fs::read(url.path())
                .await
                .inspect_err(|e| warn!(url=url_string, error=%e, "Failed to read image from dbus"))
                .ok()?;
            let content_type = match Path::new(url.path())
                .extension()
                .and_then(OsStr::to_str)
                .unwrap_or_default()
            {
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "webp" => "image/webp",
                "avif" => "image/avif",
                x => {
                    warn!(
                        url = url_string,
                        extension = x,
                        "Unknown extension in image url",
                    );
                    ""
                }
            }
            .to_owned();
            let epoch_id =
                self.image_store
                    .write()
                    .unwrap()
                    .store(*self.image_id, content_type, bytes);
            Some(ImageInfo::Internal(InternalImage {
                id: *self.image_id,
                epoch_id,
            }))
        } else {
            Some(ImageInfo::External(url_string))
        }
    }
}
