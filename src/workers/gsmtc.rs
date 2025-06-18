use crate::{
    actors::manager::{CreateModule, Manager, RemoveModule, UpdateModule},
    config::CONFIG,
    image_store::{ImageStore, SlotRef},
    model::{AlbumInfo, ImageInfo, InternalImage, ModuleState, PlayInfo, TimelineInfo},
    utilities::serde::{bool_true, deserialize_re, serialize_re},
};
use ::gsmtc::{ManagerEvent, SessionManager, SessionUpdateEvent};
use actix::Addr;
use anyhow::Result as AnyResult;
use gsmtc::{Image, PlaybackStatus, SessionModel};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tracing::{event, span, Instrument, Level};

use regex::Regex;
use std::collections::HashSet;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct GsmtcConfig {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    pub filter: GsmtcFilter,
}

impl Default for GsmtcConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            filter: GsmtcFilter::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "mode")]
pub enum GsmtcFilter {
    Include(GsmtcFilterData),
    Exclude(GsmtcFilterData),
    Disabled,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GsmtcFilterData {
    Items {
        items: HashSet<String>,
    },
    Regex {
        #[serde(serialize_with = "serialize_re", deserialize_with = "deserialize_re")]
        regex: Regex,
    },
}

impl Default for GsmtcFilter {
    fn default() -> Self {
        Self::Exclude(GsmtcFilterData::Regex {
            // the funny numbers are Firefox Desktop and Firefox Nightly
            regex: Regex::new(
                "(?i)(?:firefox|chrome|msedge|308046B0AF4A39CB|6F193CCC56814779)(?:\\.exe)?",
            )
            .unwrap(),
        })
    }
}

impl GsmtcFilter {
    pub fn pass_filter(&self, source_model_id: &str) -> bool {
        match self {
            GsmtcFilter::Include(data) => data.matches(source_model_id),
            GsmtcFilter::Exclude(data) => !data.matches(source_model_id),
            GsmtcFilter::Disabled => true,
        }
    }
}

impl GsmtcFilterData {
    pub fn matches(&self, source_model_id: &str) -> bool {
        match self {
            GsmtcFilterData::Items { items } => items.contains(source_model_id),
            GsmtcFilterData::Regex { regex } => regex.is_match(source_model_id),
        }
    }
}

#[derive(Debug)]
struct GsmtcWorker {
    manager: Addr<Manager>,
    image_store: Arc<RwLock<ImageStore>>,

    module_id: usize,
    image_id: SlotRef,

    image: Option<ImageInfo>,
    paused: bool,
}

pub async fn start_spawning(
    manager: Addr<Manager>,
    image_store: Arc<RwLock<ImageStore>>,
) -> AnyResult<()> {
    let mut rx = SessionManager::create().await?;
    tokio::spawn(
        async move {
            while let Some(evt) = rx.recv().await {
                if let ManagerEvent::SessionCreated { rx, source, .. } = evt {
                    if !CONFIG.modules.gsmtc.filter.pass_filter(&source) {
                        event!(Level::DEBUG, "Ignoring {} as it's filtered", source);
                        continue;
                    }

                    if let Ok(module_id) = manager.send(CreateModule { priority: 0 }).await {
                        event!(
                            Level::DEBUG,
                            "Creating GSMTC worker: module-id: {}",
                            module_id
                        );
                        let image_id = SlotRef::new(&image_store);
                        tokio::spawn(
                            GsmtcWorker {
                                image_id,
                                module_id,
                                image_store: image_store.clone(),
                                manager: manager.clone(),
                                image: None,
                                paused: true,
                            }
                            .feed_manager(rx)
                            .instrument(span!(
                                Level::DEBUG,
                                "GsmtcWorker",
                                id = module_id
                            )),
                        );
                    }
                }
            }
        }
        .instrument(span!(Level::INFO, "GsmtcManager")),
    );
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
    }

    #[tracing::instrument(level = "trace")]
    async fn store_image(&mut self, image: Option<Image>) -> Option<ImageInfo> {
        let mut store = self.image_store.write().unwrap();
        let img = if let Some(img) = image {
            let epoch_id = store.store(*self.image_id, img.content_type, img.data);
            Some(ImageInfo::Internal(InternalImage {
                id: *self.image_id,
                epoch_id,
            }))
        } else {
            store.clear(*self.image_id);
            None
        };
        self.image.clone_from(&img);
        img
    }

    async fn send_update(&mut self, state: ModuleState) {
        if matches!(state, ModuleState::Paused) && self.paused {
            return;
        }
        self.paused = matches!(state, ModuleState::Paused);
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
            track_number: media.track_number,
            album: media.album.map(|a| AlbumInfo {
                title: a.title,
                track_count: a.track_count,
            }),
            source: format!("gsmtc::{source}"),
            image,
            timeline: timeline
                .filter(|timeline| timeline.end > timeline.start && timeline.last_updated_at_ms > 0)
                .map(|timeline| TimelineInfo {
                    duration_ms: timeline
                        .end
                        .saturating_sub(timeline.start)
                        .as_millis()
                        .try_into()
                        .unwrap_or_default(),
                    progress_ms: timeline
                        .position
                        .saturating_sub(timeline.start)
                        .as_millis()
                        .try_into()
                        .unwrap_or_default(),
                    ts: timeline.last_updated_at_ms.try_into().unwrap_or_default(),
                    #[allow(clippy::cast_possible_truncation)]
                    rate: playback.rate as f32,
                }),
        }),
        _ => ModuleState::Paused,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_filter() {
        let filter = GsmtcFilter::default();
        let good = ["Nightly", "spotify", "VLC", "Foobar"];
        let bad = [
            "Chrome",
            "MSEdge",
            "chrome",
            "chrome.exe",
            "firefox.exe",
            "MSEdge.exe",
            "308046B0AF4A39CB",
            "6F193CCC56814779",
        ];

        for source in good {
            assert!(filter.pass_filter(source), "source={source}")
        }
        for source in bad {
            assert!(!filter.pass_filter(source), "source={source}")
        }
    }

    #[test]
    fn include_strings() {
        let filter = GsmtcFilter::Include(GsmtcFilterData::Items {
            items: HashSet::from_iter(["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]),
        });
        let good = ["foo", "bar", "baz"];
        let bad = ["Foo", "fo", "oo", "foob", "BAR", "", "something", "else"];

        for source in good {
            assert!(filter.pass_filter(source), "source={source}")
        }
        for source in bad {
            assert!(!filter.pass_filter(source), "source={source}")
        }
    }

    #[test]
    fn exclude_strings() {
        let filter = GsmtcFilter::Exclude(GsmtcFilterData::Items {
            items: HashSet::from_iter(["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]),
        });
        let good = ["Foo", "fo", "oo", "foob", "BAR", "", "something", "else"];
        let bad = ["foo", "bar", "baz"];

        for source in good {
            assert!(filter.pass_filter(source), "source={source}")
        }
        for source in bad {
            assert!(!filter.pass_filter(source), "source={source}")
        }
    }

    #[test]
    fn disabled_filter() {
        let filter = GsmtcFilter::Disabled;
        assert!(filter.pass_filter("foo"));
        assert!(filter.pass_filter(""));
        assert!(filter.pass_filter("Chrome"));
        assert!(filter.pass_filter("firefox.exe"));
    }

    #[test]
    fn filter_toml() {
        let old_default = r#"
        mode = "Exclude"
        items = ["firefox.exe", "chrome.exe", "msedge.exe"]
        "#;
        match toml::from_str::<GsmtcFilter>(old_default).unwrap() {
            GsmtcFilter::Exclude(GsmtcFilterData::Items { items }) => {
                assert_eq!(
                    items,
                    HashSet::from_iter([
                        "firefox.exe".to_owned(),
                        "chrome.exe".to_owned(),
                        "msedge.exe".to_owned()
                    ])
                );
            }
            x => assert!(false, "got={x:?}"),
        }

        let default = r#"
        mode = "Exclude"
        regex = "(?i)(?:firefox|chrome|msedge|308046B0AF4A39CB|6F193CCC56814779)(?:\\.exe)?"
        "#;
        match toml::from_str::<GsmtcFilter>(default).unwrap() {
            GsmtcFilter::Exclude(GsmtcFilterData::Regex { regex }) => {
                assert_eq!(
                    regex.as_str(),
                    "(?i)(?:firefox|chrome|msedge|308046B0AF4A39CB|6F193CCC56814779)(?:\\.exe)?"
                );
            }
            x => assert!(false, "got={x:?}"),
        }

        let include_str = r#"
        mode = "Include"
        items = ["foo", "bar"]
        "#;
        match toml::from_str::<GsmtcFilter>(include_str).unwrap() {
            GsmtcFilter::Include(GsmtcFilterData::Items { items }) => {
                assert_eq!(
                    items,
                    HashSet::from_iter(["foo".to_owned(), "bar".to_owned(),])
                );
            }
            x => assert!(false, "got={x:?}"),
        }

        let include_re = r#"
        mode = "Include"
        regex = "foo"
        "#;
        match toml::from_str::<GsmtcFilter>(include_re).unwrap() {
            GsmtcFilter::Include(GsmtcFilterData::Regex { regex }) => {
                assert_eq!(regex.as_str(), "foo");
            }
            x => assert!(false, "got={x:?}"),
        }

        let disabled = r#"
        mode = "Disabled"
        "#;
        assert!(matches!(
            toml::from_str::<GsmtcFilter>(disabled),
            Ok(GsmtcFilter::Disabled)
        ));

        let bad_re = r#"
        mode = "Include"
        regex = "("
        "#;
        match toml::from_str::<GsmtcFilter>(bad_re).unwrap() {
            GsmtcFilter::Include(GsmtcFilterData::Regex { regex }) => {
                assert_eq!(regex.as_str(), "");
            }
            x => assert!(false, "got={x:?}"),
        }
    }
}
