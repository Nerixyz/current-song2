use crate::actors::{broadcaster, ws::WsSession};
use actix_web::{get, web, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;
use tokio::sync::watch;

#[get("/client")]
async fn client(
    req: HttpRequest,
    stream: web::Payload,
    events: web::Data<watch::Receiver<broadcaster::Event>>,
) -> Result<HttpResponse> {
    ws::start(WsSession::new(events.get_ref().clone()), &req, stream)
}

pub fn init_ws(config: &mut web::ServiceConfig) {
    config.service(client);
}
