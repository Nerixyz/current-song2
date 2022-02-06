use crate::{
    actors::manager::{self, Manager},
    model::PlayInfo,
    utilities::websockets::PingingWebsocket,
};
use actix::{
    fut::ready, Actor, ActorContext, ActorFutureExt, Addr, ContextFutureSpawner, Running,
    StreamHandler, WrapFuture,
};
use actix_web_actors::{
    ws,
    ws::{Message, ProtocolError},
};
use serde::Deserialize;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{event, Level};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(40);

pub struct BrowserSession {
    hb: Instant,
    manager: Arc<Addr<Manager>>,
    id: usize,
}

impl BrowserSession {
    pub fn new(manager: Arc<Addr<Manager>>) -> Self {
        Self {
            hb: Instant::now(),
            manager,
            id: 0,
        }
    }
}

impl Actor for BrowserSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.manager
            .send(manager::CreateModule { priority: 1 })
            .into_actor(self)
            .then(|res: Result<usize, _>, this, ctx| {
                match res {
                    Ok(id) => this.id = id,
                    Err(e) => {
                        event!(Level::WARN, error = %e, "Failed creating module");
                        ctx.stop();
                    }
                };
                ready(())
            })
            .wait(ctx);

        self.init_hb_check(ctx, HEARTBEAT_INTERVAL, CLIENT_TIMEOUT);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.manager.do_send(manager::RemoveModule { id: self.id });

        Running::Stop
    }
}

impl PingingWebsocket for BrowserSession {
    fn last_hb(&self) -> Instant {
        self.hb
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for BrowserSession {
    fn handle(&mut self, item: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Text(text)) => {
                let msg = serde_json::from_str::<Response>(&text);
                match msg {
                    Ok(Response::Pong) => self.hb = Instant::now(),
                    Ok(Response::Inactive) => {
                        self.manager.do_send(manager::UpdateModule::paused(self.id));
                    }
                    Ok(Response::Active(info)) => {
                        self.manager
                            .do_send(manager::UpdateModule::playing(self.id, info));
                    }
                    Err(e) => {
                        event!(Level::WARN, id = %self.id, error = %e, "Invalid WS message");
                    }
                }
            }
            Ok(_) => (),
            Err(e) => {
                event!(Level::WARN, id = %self.id, error = %e, "WebSocket error");
                ctx.stop();
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "data")]
enum Response {
    Pong,
    Active(PlayInfo),
    Inactive,
}
