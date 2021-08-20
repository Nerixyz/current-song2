use crate::actors::{manager, manager::Update};
use actix::{Actor, Context, Handler};
use tokio::sync::watch;

pub type Event = String;

pub struct Broadcaster {
    event_tx: watch::Sender<Event>,
}

impl Broadcaster {
    pub fn new(event_tx: watch::Sender<Event>) -> Self {
        Self { event_tx }
    }
}

impl Actor for Broadcaster {
    type Context = Context<Self>;
}

impl Handler<manager::Update> for Broadcaster {
    type Result = ();

    fn handle(&mut self, msg: Update, _ctx: &mut Self::Context) -> Self::Result {
        if let Err(e) = self.event_tx.send(msg.0) {
            log::warn!("Could not send update message: {}", e);
        }
    }
}
