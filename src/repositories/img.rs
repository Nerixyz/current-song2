use crate::image_store::ImageStore;
use actix_web::{error, get, web, HttpResponse, Result};
use std::sync::RwLock;

#[get("/{id}/{target_epoch}")]
async fn get_image(
    path: web::Path<(usize, usize)>,
    store: web::Data<RwLock<ImageStore>>,
) -> Result<HttpResponse> {
    let (id, target_epoch) = path.into_inner();
    let image_store = store.into_inner();
    let store = image_store.read().unwrap();
    let img = store.get(id, target_epoch);
    if let Some(img) = img {
        Ok(HttpResponse::Ok()
            .content_type(img.content_type.as_str())
            .body(img.data.clone()))
    } else {
        Err(error::ErrorNotFound("Requested image doesn't exist"))
    }
}

pub fn init_img(config: &mut web::ServiceConfig) {
    config.service(get_image);
}
