#![allow(clippy::unused_async)] // required by the actix macros

use crate::{
    actors::{client_ws, extension_ws, manager::Manager},
    manager,
};
use actix::Addr;
use actix_web::{get, web, HttpRequest, HttpResponse, Result};
use tokio::sync::watch;
use tracing::{event, Level};

#[get("/client")]
async fn client(
    req: HttpRequest,
    stream: web::Payload,
    events: web::Data<watch::Receiver<manager::Event>>,
) -> Result<HttpResponse> {
    event!(Level::DEBUG, "Client connected");
    let (response, session, messages) = actix_ws::handle(&req, stream)?;
    actix_web::rt::spawn(client_ws::handle(
        session,
        messages.aggregate_continuations(),
        events.as_ref().clone(),
    ));

    Ok(response)
}

#[get("/extension")]
async fn extension(
    req: HttpRequest,
    stream: web::Payload,
    manager: web::Data<Addr<Manager>>,
) -> Result<HttpResponse> {
    event!(Level::DEBUG, "Extension connected");
    let (response, session, messages) = actix_ws::handle(&req, stream)?;
    actix_web::rt::spawn(extension_ws::handle(
        session,
        messages.aggregate_continuations(),
        manager.into_inner(),
    ));

    Ok(response)
}

pub fn init_ws(config: &mut web::ServiceConfig) {
    config.service(client).service(extension);
}
