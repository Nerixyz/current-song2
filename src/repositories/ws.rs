use crate::{
    actors::{browser::BrowserSession, manager::Manager, ws::WsSession},
    manager,
};
use actix::Addr;
use actix_web::{get, web, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;
use tokio::sync::watch;
use tracing::{event, Level};

#[get("/client")]
async fn client(
    req: HttpRequest,
    stream: web::Payload,
    events: web::Data<watch::Receiver<manager::Event>>,
) -> Result<HttpResponse> {
    event!(Level::DEBUG, "Client connected");
    ws::start(WsSession::new(events.get_ref().clone()), &req, stream)
}

#[get("/extension")]
async fn extension(
    req: HttpRequest,
    stream: web::Payload,
    manager: web::Data<Addr<Manager>>,
) -> Result<HttpResponse> {
    event!(Level::DEBUG, "Extension connected");
    ws::start(BrowserSession::new(manager.into_inner()), &req, stream)
}

pub fn init_ws(config: &mut web::ServiceConfig) {
    config.service(client).service(extension);
}
