mod img;
mod ws;

use actix_cors::Cors;
use actix_web::{middleware::Compat, web};

pub fn init_repositories(config: &mut web::ServiceConfig) {
    config
        .service(web::scope("/img").configure(img::init_img))
        .service(
            web::scope("/ws")
                .wrap(Compat::new(Cors::permissive()))
                .configure(ws::init_ws),
        );
}
