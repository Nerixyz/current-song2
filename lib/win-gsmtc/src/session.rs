use crate::{
    model::{Image, MediaModel, PlaybackStatus, SessionModel, TimelineModel},
    util::{optional_result, request_media_properties},
};
use std::{
    convert::TryInto,
    sync::{Arc, Weak},
};
use tokio::sync::mpsc;
use tracing::{debug, event, Level};
use windows::{
    core::{AgileReference, Result},
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Media::Control::GlobalSystemMediaTransportControlsSession,
};

pub struct SessionHandle {
    pub id: usize,
    pub sender: Arc<mpsc::UnboundedSender<SessionCommand>>,
}

struct SessionWorker {
    model: SessionModel,

    session: GlobalSystemMediaTransportControlsSession,

    loop_tx: Weak<mpsc::UnboundedSender<SessionCommand>>,
    loop_rx: mpsc::UnboundedReceiver<SessionCommand>,

    sess_tx: mpsc::UnboundedSender<SessionUpdateEvent>,

    playback_token: EventRegistrationToken,
    media_token: EventRegistrationToken,
    timeline_token: EventRegistrationToken,
}

#[derive(Debug)]
pub enum SessionUpdateEvent {
    Model(SessionModel),
    Media(SessionModel, Option<Image>),
}

#[derive(Debug)]
pub enum SessionCommand {
    PlaybackInfoChanged,
    MediaPropertiesChanged,
    MediaPropertiesResult(MediaModel, Option<Image>),
    TimelinePropertiesChanged,
    Close,
}

impl SessionHandle {
    pub fn create(
        id: usize,
        sess: GlobalSystemMediaTransportControlsSession,
        sess_tx: mpsc::UnboundedSender<SessionUpdateEvent>,
    ) -> Result<(Self, String)> {
        let (loop_tx, loop_rx) = mpsc::unbounded_channel();
        let loop_tx = Arc::new(loop_tx);
        let playback_token = {
            let loop_tx = Arc::downgrade(&loop_tx);
            sess.PlaybackInfoChanged(TypedEventHandler::new(move |_, _| {
                if let Some(loop_tx) = loop_tx.upgrade() {
                    loop_tx.send(SessionCommand::PlaybackInfoChanged).ok();
                }
                Ok(())
            }))?
        };
        let media_token = {
            let loop_tx = Arc::downgrade(&loop_tx);
            sess.MediaPropertiesChanged(TypedEventHandler::new(move |_, _| {
                if let Some(loop_tx) = loop_tx.upgrade() {
                    loop_tx.send(SessionCommand::MediaPropertiesChanged).ok();
                }
                Ok(())
            }))?
        };
        let timeline_token = {
            let loop_tx = Arc::downgrade(&loop_tx);
            sess.TimelinePropertiesChanged(TypedEventHandler::new(move |_, _| {
                if let Some(loop_tx) = loop_tx.upgrade() {
                    loop_tx.send(SessionCommand::TimelinePropertiesChanged).ok();
                }
                Ok(())
            }))?
        };

        let source = sess.SourceAppUserModelId()?.to_string();
        SessionWorker {
            session: sess,
            model: SessionModel {
                playback: None,
                timeline: None,
                media: None,
                source: source.clone(),
            },

            loop_tx: Arc::downgrade(&loop_tx),
            loop_rx,
            sess_tx,

            playback_token,
            media_token,
            timeline_token,
        }
        .spawn();

        Ok((
            Self {
                sender: loop_tx,
                id,
            },
            source,
        ))
    }
}

impl SessionWorker {
    fn spawn(self) {
        tokio::spawn(self.run());
    }

    async fn run(mut self) {
        if let Some(loop_tx) = self.loop_tx.upgrade() {
            loop_tx.send(SessionCommand::PlaybackInfoChanged).ok();
            loop_tx.send(SessionCommand::TimelinePropertiesChanged).ok();
            loop_tx.send(SessionCommand::MediaPropertiesChanged).ok();
        }

        while let Some(cmd) = self.loop_rx.recv().await {
            match self.handle_command(cmd) {
                Err(e) => {
                    event!(Level::WARN, error = %e, source = %self.model.source, "Could not handle command")
                }
                Ok(false) => break,
                _ => (),
            }
        }
    }

    /// Returns Result<running>
    fn handle_command(&mut self, cmd: SessionCommand) -> Result<bool> {
        event!(Level::TRACE, source = %self.model.source, command = ?cmd);
        match cmd {
            SessionCommand::PlaybackInfoChanged => {
                let model = optional_result(self.session.GetPlaybackInfo()?.try_into())?;
                if !(model.is_none() == self.model.playback.is_none()) {
                    self.model.playback = model;

                    self.sess_tx
                        .send(SessionUpdateEvent::Model(self.model.clone()))
                        .ok();
                }
            }
            SessionCommand::MediaPropertiesChanged => {
                let loop_tx = self.loop_tx.clone();
                let session = AgileReference::new(&self.session)?;
                tokio::spawn(async move {
                    match request_media_properties(loop_tx, session).await {
                        Ok(None) => debug!("Empty media properties"),
                        Err(e) => event!(Level::WARN, error = %e, "Could not get media properties"),
                        _ => (),
                    }
                });
            }
            SessionCommand::TimelinePropertiesChanged => {
                let model = self.session.GetTimelineProperties()?.try_into()?;
                if !timeline_actually_the_same(&self.model.timeline, &model) {
                    let should_skip = skip_timeline_emit(&self.model, &model);
                    self.model.timeline = Some(model);

                    if !should_skip {
                        self.sess_tx
                            .send(SessionUpdateEvent::Model(self.model.clone()))
                            .ok();
                    }
                }
            }
            SessionCommand::MediaPropertiesResult(media, image) => {
                self.model.media = Some(media);
                self.sess_tx
                    .send(SessionUpdateEvent::Media(self.model.clone(), image))
                    .ok();
            }
            SessionCommand::Close => return Ok(false),
        };

        Ok(true)
    }
}

fn timeline_actually_the_same(first: &Option<TimelineModel>, second: &TimelineModel) -> bool {
    first
        .as_ref()
        .map(|first| {
            first.eq(second)
                || (first.start == second.start
                    && first.end == second.end
                    && rough_eq(
                        second.last_updated_at_ms - first.last_updated_at_ms,
                        (second.position - first.position) / 10_000,
                    ))
        })
        .unwrap_or_default()
}

#[inline]
fn rough_eq(a: i64, b: i64) -> bool {
    // either: a = b - 1 so b = a + 1
    // or: a = b + 1 so b = a - 1
    // or: a == b
    a == b || (a < b + 2 && a > b - 2)
}

fn skip_timeline_emit(model: &SessionModel, new: &TimelineModel) -> bool {
    model
        .playback
        .as_ref()
        .map(|playback| {
            playback.status == PlaybackStatus::Paused
                && model
                    .timeline
                    .as_ref()
                    .map(|old| {
                        old.start == new.start && old.end == new.end && old.position == new.position
                    })
                    .unwrap_or_default()
        })
        .unwrap_or_default()
}

impl Drop for SessionWorker {
    fn drop(&mut self) {
        self.session
            .RemovePlaybackInfoChanged(&self.playback_token)
            .ok();
        self.session
            .RemoveTimelinePropertiesChanged(&self.timeline_token)
            .ok();
        self.session
            .RemoveMediaPropertiesChanged(&self.media_token)
            .ok();
    }
}
