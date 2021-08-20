use crate::actors::broadcaster;
use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web_actors::{
    ws,
    ws::{Message, ProtocolError},
};
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(40);

pub struct WsSession {
    hb: Instant,
    rx: Option<watch::Receiver<broadcaster::Event>>,
}

impl WsSession {
    pub fn new(rx: watch::Receiver<broadcaster::Event>) -> Self {
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

        ctx.run_interval(HEARTBEAT_INTERVAL, |this, ctx| {
            if Instant::now().duration_since(this.hb) > CLIENT_TIMEOUT {
                ctx.stop();
            } else {
                ctx.text(serde_json::json!({ "type": "Ping" }).to_string());
            }
        });
    }
}

impl StreamHandler<String> for WsSession {
    fn handle(&mut self, item: String, ctx: &mut Self::Context) {
        ctx.text(item);
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
}
