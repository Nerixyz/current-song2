use crate::{
    actors::manager::{CreateModule, Manager, RemoveModule, UpdateModule},
    config::CONFIG,
    image_store::ImageStore,
    model::{ImageInfo, InternalImage, ModuleState, PlayInfo, TimelineInfo},
};
use ::gsmtc::{ManagerEvent, SessionManager, SessionUpdateEvent};
use actix::Addr;
use anyhow::Result as AnyResult;
use futures::future;
use gsmtc::{Image, PlaybackStatus, SessionModel};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{event, Level};

struct GsmtcWorker {
    manager: Addr<Manager>,
    image_store: Arc<RwLock<ImageStore>>,

    module_id: usize,
    image_id: usize,

    image: Option<ImageInfo>,
    paused: bool,
}

pub async fn start_spawning(
    manager: Addr<Manager>,
    image_store: Arc<RwLock<ImageStore>>,
) -> AnyResult<()> {
    let mut rx = SessionManager::new().await?;
    tokio::spawn(async move {
        while let Some(evt) = rx.recv().await {
            if let ManagerEvent::SessionCreated { rx, source, .. } = evt {
                if !CONFIG.modules.gsmtc.filter.pass_filter(&source) {
                    event!(Level::DEBUG, "Ignoring {} as it's filtered", source);
                    continue;
                }

                if let (Ok(module_id), mut store) = future::join(
                    manager.send(CreateModule { priority: 0 }),
                    image_store.write(),
                )
                .await
                {
                    event!(
                        Level::DEBUG,
                        "Creating GSMTC worker: module-id: {}",
                        module_id
                    );
                    tokio::spawn(
                        GsmtcWorker {
                            image_id: store.create_id(),
                            module_id,
                            image_store: image_store.clone(),
                            manager: manager.clone(),
                            image: None,
                            paused: true,
                        }
                        .feed_manager(rx),
                    );
                }
            }
        }
    });
    Ok(())
}

impl GsmtcWorker {
    async fn feed_manager(mut self, mut rx: mpsc::UnboundedReceiver<SessionUpdateEvent>) {
        while let Some(evt) = rx.recv().await {
            match evt {
                SessionUpdateEvent::Model(model) => {
                    self.send_update(convert_model(model, self.image.clone()))
                        .await;
                }
                SessionUpdateEvent::Media(model, image) => {
                    let img = self.store_image(image).await;
                    self.send_update(convert_model(model, img)).await;
                }
            }
        }
        self.manager
            .send(RemoveModule { id: self.module_id })
            .await
            .ok();

        self.image_store.write().await.remove(self.image_id);
    }

    async fn store_image(&mut self, image: Option<Image>) -> Option<ImageInfo> {
        let mut store = self.image_store.write().await;
        let img = match image {
            Some(img) => {
                let epoch_id = store.store(self.image_id, img.content_type, img.data);
                Some(ImageInfo::Internal(InternalImage {
                    id: self.image_id,
                    epoch_id,
                }))
            }
            None => {
                store.clear(self.image_id);
                None
            }
        };
        self.image = img.clone();
        img
    }

    async fn send_update(&mut self, state: ModuleState) {
        if matches!(state, ModuleState::Paused) && self.paused {
            return;
        }
        self.paused = matches!(state, ModuleState::Paused);
        self.manager
            .send(UpdateModule {
                id: self.module_id,
                state,
            })
            .await
            .ok();
    }
}

fn convert_model(from: SessionModel, image: Option<ImageInfo>) -> ModuleState {
    match from {
        SessionModel {
            playback: Some(playback),
            media: Some(media),
            timeline,
            source,
        } if playback.status == PlaybackStatus::Playing => ModuleState::Playing(PlayInfo {
            title: media.title,
            artist: media.artist,
            source: format!("gsmtc::{}", source),
            image,
            timeline: timeline
                .filter(|timeline| timeline.end > timeline.start && timeline.last_updated_at_ms > 0)
                .map(|timeline| TimelineInfo {
                    duration_ms: ((timeline.end - timeline.start) / 10_000) as u64,
                    progress_ms: ((timeline.position - timeline.start) / 10_000) as u64,
                    ts: timeline.last_updated_at_ms as u64,
                    rate: playback.rate as f32,
                }),
        }),
        _ => ModuleState::Paused,
    }
}
