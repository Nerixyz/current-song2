use crate::{
    actors::manager::{self, Manager},
    model::{ModuleState, PlayInfo},
};
use actix::{
    fut::ready, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner,
    Running, StreamHandler, WrapFuture,
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
                        log::warn!("Failed creating module: {}", e);
                        ctx.stop();
                    }
                };
                ready(())
            })
            .wait(ctx);

        ctx.run_interval(HEARTBEAT_INTERVAL, |this, ctx| {
            if Instant::now().duration_since(this.hb) > CLIENT_TIMEOUT {
                ctx.stop();
            } else {
                ctx.text(serde_json::json!({ "type": "Ping" }).to_string());
            }
        });
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.manager.do_send(manager::RemoveModule { id: self.id });

        Running::Stop
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
                        self.manager.do_send(manager::UpdateModule {
                            id: self.id,
                            state: ModuleState::Paused,
                        });
                    }
                    Ok(Response::Active(info)) => {
                        self.manager.do_send(manager::UpdateModule {
                            id: self.id,
                            state: ModuleState::Playing(info),
                        });
                    }
                    Err(e) => {
                        log::warn!("Invalid WS message: {}", e);
                    }
                }
            }
            Ok(_) => (),
            Err(e) => {
                log::warn!("Websocket error: {}", e);
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
