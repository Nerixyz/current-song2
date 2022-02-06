use crate::{manager, utilities::websockets::PingingWebsocket};
use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web_actors::{
    ws,
    ws::{Message, ProtocolError},
};
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;
use tracing::{error, event, Level};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(40);

pub struct WsSession {
    hb: Instant,
    rx: Option<watch::Receiver<manager::Event>>,
}

impl WsSession {
    pub fn new(rx: watch::Receiver<manager::Event>) -> Self {
        Self {
            hb: Instant::now(),
            rx: Some(rx),
        }
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.add_stream(WatchStream::new(self.rx.take().unwrap()));
        self.init_hb_check(ctx, HEARTBEAT_INTERVAL, CLIENT_TIMEOUT);
    }
}

impl PingingWebsocket for WsSession {
    fn last_hb(&self) -> Instant {
        self.hb
    }
}

impl StreamHandler<manager::Event> for WsSession {
    fn handle(&mut self, item: manager::Event, ctx: &mut Self::Context) {
        match serde_json::to_string(&*item) {
            Ok(json) => ctx.text(json),
            Err(e) => error!(error=%e, "Cannot serialize json"),
        }
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        ctx.stop();
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, item: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Text(text)) => {
                let msg = serde_json::from_str::<Response>(&text);
                if let Ok(Response::Pong) = msg {
                    self.hb = Instant::now();
                }
            }
            Ok(_) => (),
            Err(e) => {
                event!(Level::WARN, error = %e, "WebSocket error");
                ctx.stop();
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "data")]
enum Response {
    Pong,
}
