mod img;
mod ws;

use actix_cors::Cors;
use actix_web::web;

pub fn init_repositories(config: &mut web::ServiceConfig) {
    config
        .service(web::scope("/img").configure(img::init_img))
        .service(web::scope("/ws").wrap(Cors::permissive()).configure(ws::init_ws));
}
