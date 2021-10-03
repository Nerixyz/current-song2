use crate::actors::{browser::BrowserSession, manager::Manager};
use actix::Addr;
use actix_web::{get, web, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;

#[get("/ws")]
async fn browser_ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    manager: web::Data<Addr<Manager>>,
) -> Result<HttpResponse> {
    ws::start(BrowserSession::new(manager.into_inner()), &req, stream)
}

pub fn init_browser(config: &mut web::ServiceConfig) {
    config.service(browser_ws_handler);
}
