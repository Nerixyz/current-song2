mod messages;
use crate::model::ModuleState;
use actix::{Actor, ActorFutureExt, Context, ContextFutureSpawner, Handler, Recipient, WrapFuture};
use futures::future;
pub use messages::*;
use std::collections::HashMap;

struct Module {
    priority: u8,
    state: ModuleState,
}

pub struct Manager {
    receiver: Recipient<Update>,

    modules: HashMap<usize, Module>,
    current_module: Option<usize>,

    next_id: usize,
}

impl Manager {
    pub fn new(receiver: Recipient<Update>) -> Self {
        Self {
            receiver,
            modules: Default::default(),
            current_module: None,
            next_id: 0,
        }
    }

    fn send_update_state(&mut self, ctx: &mut <Self as Actor>::Context, updated: usize) {
        if let Ok(Some(state)) = self.update_state(updated) {
            self.receiver
                .send(Update(state))
                .into_actor(self)
                .then(|_, _, _| future::ready(()))
                .spawn(ctx);
        }
    }

    fn update_state(&mut self, updated: usize) -> anyhow::Result<Option<String>> {
        let mut active: Vec<(usize, &Module)> = self
            .modules
            .iter()
            .filter_map(|(id, m)| match m {
                Module {
                    state: ModuleState::Playing(_),
                    ..
                } => Some((*id, m)),
                _ => None,
            })
            .collect();

        Ok(if active.is_empty() {
            if self.current_module.is_none() {
                None
            } else {
                self.current_module = None;
                log::debug!("Send: Paused");
                Some(serde_json::to_string(&ModuleState::Paused)?)
            }
        } else {
            active.sort_by(|(_, a), (_, b)| a.priority.cmp(&b.priority));
            let (id, module) = active.get(0).unwrap(); // we checked if the vec was empty

            // if the current module didn't change and the updated module was not the current one
            // -> this would result in no change -> no update
            if self
                .current_module
                .as_ref()
                .map(|current| current == id && updated != *current)
                .unwrap_or(false)
            {
                return Ok(None);
            }

            self.current_module = Some(*id);

            log::debug!("Send: {:?}", module.state);
            Some(serde_json::to_string(&module.state)?)
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
                state: ModuleState::Paused,
            },
        );
        id
    }
}

impl Handler<UpdateModule> for Manager {
    type Result = ();

    fn handle(&mut self, msg: UpdateModule, ctx: &mut Self::Context) -> Self::Result {
        let current_priority = self
            .current_module
            .as_ref()
            .and_then(|id| self.modules.get(id).map(|current| current.priority));

        if let Some(module) = self.modules.get_mut(&msg.id) {
            log::debug!(
                "Update {} - {:?} current-prio: {:?} mod-prio: {}",
                msg.id,
                msg.state,
                current_priority,
                module.priority
            );
            module.state = msg.state;

            if current_priority
                .map(|current_priority| module.priority >= current_priority)
                .unwrap_or(true)
            {
                self.send_update_state(ctx, msg.id);
            }
        }
    }
}

impl Handler<RemoveModule> for Manager {
    type Result = ();

    fn handle(&mut self, msg: RemoveModule, ctx: &mut Self::Context) -> Self::Result {
        if self.modules.remove(&msg.id).is_some() && self.current_module == Some(msg.id) {
            self.send_update_state(ctx, msg.id);
        }
    }
}
