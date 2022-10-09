use crate::{
    session::{SessionCommand, SessionHandle, SessionUpdateEvent},
    util::ResultExt,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tracing::{event, Level};
use windows::{
    core::Result,
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Media::Control::{
        GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    },
};

pub struct SessionManager {
    sessions: HashMap<String, SessionHandle>,
    next_session_id: usize,

    manager: GlobalSystemMediaTransportControlsSessionManager,

    event_tx: mpsc::UnboundedSender<ManagerEvent>,

    loop_tx: Arc<mpsc::UnboundedSender<ManagerCommand>>,
    loop_rx: mpsc::UnboundedReceiver<ManagerCommand>,

    changed_token: EventRegistrationToken,
    current_changed_token: EventRegistrationToken,
}

#[derive(Debug)]
pub enum ManagerEvent {
    SessionCreated {
        session_id: usize,
        rx: mpsc::UnboundedReceiver<SessionUpdateEvent>,
        source: String,
    },
    SessionRemoved {
        session_id: usize,
    },
    /// We don't use this event internally.
    /// We compute the current session by the last event
    /// that was received for a session.
    CurrentSessionChanged {
        session_id: Option<usize>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ManagerCommand {
    UpdateSessions,
    CurrentSessionChanged,
}

impl SessionManager {
    pub async fn new() -> Result<mpsc::UnboundedReceiver<ManagerEvent>> {
        let this = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.await?;

        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (loop_tx, loop_rx) = mpsc::unbounded_channel();
        let loop_tx = Arc::new(loop_tx);

        let update_token = {
            let loop_tx = Arc::downgrade(&loop_tx);
            this.SessionsChanged(&TypedEventHandler::new(move |_, _| {
                event!(Level::DEBUG, "SessionsChanged");
                if let Some(loop_tx) = loop_tx.upgrade() {
                    loop_tx.send(ManagerCommand::UpdateSessions).ok();
                }
                Ok(())
            }))?
        };
        let current_changed_token = {
            let loop_tx = Arc::downgrade(&loop_tx);
            this.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
                event!(Level::DEBUG, "Current SessionChanged");
                if let Some(loop_tx) = loop_tx.upgrade() {
                    loop_tx.send(ManagerCommand::CurrentSessionChanged).ok();
                }
                Ok(())
            }))?
        };

        SessionManager {
            sessions: Default::default(),
            next_session_id: 0,
            manager: this,
            event_tx,
            loop_tx,
            loop_rx,

            changed_token: update_token,
            current_changed_token,
        }
        .spawn();

        Ok(event_rx)
    }

    fn spawn(self) {
        tokio::spawn(self.run());
    }

    async fn run(mut self) {
        self.loop_tx.send(ManagerCommand::UpdateSessions).ok();
        self.loop_tx
            .send(ManagerCommand::CurrentSessionChanged)
            .ok();
        while let Some(cmd) = self.loop_rx.recv().await {
            if let Err(e) = self.handle_command(cmd) {
                event!(Level::ERROR, error = %e, command = ?cmd, "Manager encountered error - exiting");
                break;
            }
        }
        event!(Level::INFO, "Manager Loop Ended")
    }

    fn handle_command(&mut self, cmd: ManagerCommand) -> Result<()> {
        match cmd {
            ManagerCommand::UpdateSessions => {
                let updated: Result<HashMap<String, GlobalSystemMediaTransportControlsSession>> =
                    self.manager
                        .GetSessions()?
                        .into_iter()
                        .map(|session| Ok((session.SourceAppUserModelId()?.to_string(), session)))
                        .collect();
                let mut updated = updated?;

                let to_remove: Vec<(String, usize)> = self
                    .sessions
                    .iter()
                    .filter_map(|(k, session)| {
                        if updated.remove(k).is_some() {
                            None
                        } else {
                            Some((k.clone(), session.id))
                        }
                    })
                    .collect();

                event!(Level::DEBUG, "Update: remove {} sessions", to_remove.len());

                for (session, id) in to_remove {
                    self.event_tx
                        .send(ManagerEvent::SessionRemoved { session_id: id })
                        .ok();
                    if let Some(session) = self.sessions.remove(&session) {
                        session.sender.send(SessionCommand::Close).ok();
                    }
                }

                for (model_id, to_create) in updated.into_iter() {
                    self.create_session(model_id, to_create)?;
                }
            }
            ManagerCommand::CurrentSessionChanged => {
                match self.manager.GetCurrentSession().opt() {
                    // There's a new session
                    Ok(Some(current)) => {
                        match self
                            .sessions
                            .get(&current.SourceAppUserModelId()?.to_string())
                        {
                            // We have the session saved
                            Some(session) => {
                                self.event_tx
                                    .send(ManagerEvent::CurrentSessionChanged {
                                        session_id: Some(session.id),
                                    })
                                    .ok();
                            }
                            // It's a new session
                            None => {
                                let session_id = self.create_session(
                                    current.SourceAppUserModelId()?.to_string(),
                                    current,
                                )?;
                                self.event_tx
                                    .send(ManagerEvent::CurrentSessionChanged {
                                        session_id: Some(session_id),
                                    })
                                    .ok();
                            }
                        }
                    }
                    // There's no current session
                    Ok(None) => {
                        self.event_tx
                            .send(ManagerEvent::CurrentSessionChanged { session_id: None })
                            .ok();
                    }
                    Err(e) => {
                        event!(Level::WARN, error = %e, "Could not get current session");
                    }
                };
            }
        }
        Ok(())
    }

    fn create_session(
        &mut self,
        model_id: String,
        session: GlobalSystemMediaTransportControlsSession,
    ) -> Result<usize> {
        let id = self.next_session_id;
        self.next_session_id = self.next_session_id.overflowing_add(1).0;

        let (tx, rx) = mpsc::unbounded_channel();

        let (session, source) = SessionHandle::create(id, session, tx)?;
        self.sessions.insert(model_id, session);

        event!(Level::DEBUG, id, %source, "New Session");

        self.event_tx
            .send(ManagerEvent::SessionCreated {
                session_id: id,
                rx,
                source,
            })
            .ok();

        Ok(id)
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        self.manager.RemoveSessionsChanged(self.changed_token).ok();
        self.manager
            .RemoveCurrentSessionChanged(self.current_changed_token)
            .ok();
    }
}
