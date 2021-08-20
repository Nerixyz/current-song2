mod actors;
mod image_store;
mod model;
mod repositories;
mod workers;

use crate::{
    actors::{broadcaster, broadcaster::Broadcaster, manager::Manager},
    image_store::ImageStore,
    repositories::init_repositories,
};
use actix::Actor;
use actix_web::{web, App, HttpServer};
use tokio::sync::{watch, RwLock};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let (event_tx, event_rx) =
        watch::channel::<broadcaster::Event>(serde_json::json!({"type": "Paused"}).to_string());

    let broadcaster = Broadcaster::new(event_tx).start();
    let manager = Manager::new(broadcaster.recipient()).start();

    let image_store = web::Data::new(RwLock::new(ImageStore::new()));

    if cfg!(windows) {
        workers::gsmtc::start_spawning(manager, image_store.clone().into_inner())
            .await
            .unwrap();
    }

    let event_rx = web::Data::new(event_rx);
    HttpServer::new(move || {
        App::new()
            .app_data(event_rx.clone())
            .app_data(image_store.clone())
            .service(web::scope("api").configure(init_repositories))
            .service(actix_files::Files::new("/", "client/dist").index_file("index.html"))
    })
    .bind("127.0.0.1:48457")?
    .run()
    .await
}
