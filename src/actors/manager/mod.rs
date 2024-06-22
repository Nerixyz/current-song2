mod messages;
#[cfg(test)]
mod tests;

use crate::model::ModuleState;
use actix::{Actor, Context, Handler};
pub use messages::*;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::watch;
use tracing::{error, event, Level};

pub type Event = Arc<ModuleState>;

#[derive(Debug)]
struct Module {
    priority: u8,
    state: Arc<ModuleState>,
}

#[derive(Debug)]
pub struct Manager {
    event_tx: watch::Sender<Event>,

    modules: HashMap<usize, Module>,
    current_module: Option<usize>,

    next_id: usize,
}

impl Manager {
    pub fn new(event_tx: watch::Sender<Event>) -> Self {
        Self {
            event_tx,
            modules: HashMap::default(),
            current_module: None,
            next_id: 0,
        }
    }

    fn send_update_state(&mut self, updated: usize) {
        if let Ok(Some(state)) = self.update_state(updated) {
            if let Err(e) = self.event_tx.send(state) {
                error!(error = %e,"Couldn't send state on event_tx");
            }
        }
    }

    /// Updates the state of a module.
    /// Returns
    /// * `Ok(Some(..))` if a new state has to be sent
    /// * `Ok(None)`     is nothing changed
    fn update_state(&mut self, updated: usize) -> anyhow::Result<Option<Event>> {
        let mut active: Vec<(usize, &Module)> = self
            .modules
            .iter()
            .filter_map(|(id, m)| match *m.state {
                ModuleState::Playing(_) => Some((*id, m)),
                ModuleState::Paused => None,
            })
            .collect();

        Ok(if active.is_empty() {
            if self.current_module.is_none() {
                None
            } else {
                self.current_module = None;
                event!(Level::DEBUG, id = updated, message = ?(ModuleState::Paused), "Send");
                Some(Arc::new(ModuleState::Paused))
            }
        } else {
            // sort descending
            active.sort_by(|(_, a), (_, b)| b.priority.cmp(&a.priority));
            let (id, module) = active.first().unwrap(); // we checked if the vec was empty

            // if the current module didn't change and the updated module was not the current one
            // -> this would result in no change -> no update
            if self
                .current_module
                .as_ref()
                .map_or(false, |current| current == id && updated != *current)
            {
                return Ok(None);
            }

            self.current_module = Some(*id);

            event!(Level::DEBUG, id = updated, message = ?(module.state), "Send");
            Some(module.state.clone())
        })
    }
}

impl Actor for Manager {
    type Context = Context<Self>;
}

impl Handler<CreateModule> for Manager {
    type Result = usize;

    fn handle(&mut self, msg: CreateModule, _: &mut Self::Context) -> Self::Result {
        let id = self.next_id;
        self.next_id = self.next_id.overflowing_add(1).0;
        self.modules.insert(
            id,
            Module {
                priority: msg.priority,
                state: Arc::new(ModuleState::Paused),
            },
        );
        id
    }
}

impl Handler<UpdateModule> for Manager {
    type Result = ();

    fn handle(&mut self, msg: UpdateModule, _: &mut Self::Context) -> Self::Result {
        let current_priority = self
            .current_module
            .as_ref()
            .and_then(|id| self.modules.get(id).map(|current| current.priority));

        if let Some(module) = self.modules.get_mut(&msg.id) {
            event!(Level::DEBUG,
                id = msg.id,
                state = ?msg.state,
                current_priority = ?current_priority,
                module.priority = module.priority,  "Update");
            module.state = Arc::new(msg.state);

            if current_priority.map_or(true, |current_priority| module.priority >= current_priority)
            {
                self.send_update_state(msg.id);
            }
        }
    }
}

impl Handler<RemoveModule> for Manager {
    type Result = ();

    fn handle(&mut self, msg: RemoveModule, _: &mut Self::Context) -> Self::Result {
        if self.modules.remove(&msg.id).is_some() && self.current_module == Some(msg.id) {
            self.send_update_state(msg.id);
        }
    }
}
