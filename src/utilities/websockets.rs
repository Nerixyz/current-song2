use actix::{Actor, ActorContext, AsyncContext};
use actix_web_actors::ws::WebsocketContext;
use std::time::{Duration, Instant};

pub trait PingingWebsocket {
    fn last_hb(&self) -> Instant;

    fn init_hb_check(
        &self,
        ctx: &mut WebsocketContext<Self>,
        heartbeat_interval: Duration,
        client_timeout: Duration,
    ) where
        Self: Actor<Context = WebsocketContext<Self>>,
    {
        ctx.run_interval(heartbeat_interval, move |this, ctx| {
            if Instant::now().duration_since(this.last_hb()) > client_timeout {
                ctx.stop();
            } else {
                ctx.text(serde_json::json!({ "type": "Ping" }).to_string());
            }
        });
    }
}
