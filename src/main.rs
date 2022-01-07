#![windows_subsystem = "windows"]

mod actors;
mod config;
mod image_store;
mod logging;
mod model;
mod repositories;
mod static_files;
mod win_setup;
mod workers;
#[macro_use]
mod macros;

use config::CONFIG;

use crate::{
    actors::{broadcaster, broadcaster::Broadcaster, manager::Manager},
    image_store::ImageStore,
    logging::init_logging,
    repositories::init_repositories,
};
use actix::Actor;
use actix_web::{web, App, HttpServer};
use tokio::sync::{watch, RwLock};
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn async_main() -> std::io::Result<()> {
    let (event_tx, event_rx) =
        watch::channel::<broadcaster::Event>(serde_json::json!({"type": "Paused"}).to_string());

    let broadcaster = Broadcaster::new(event_tx).start();
    let manager = Manager::new(broadcaster.recipient()).start();

    let image_store = web::Data::new(RwLock::new(ImageStore::new()));

    if cfg!(windows) {
        if CONFIG.modules.gsmtc.enabled {
            workers::gsmtc::start_spawning(manager.clone(), image_store.clone().into_inner())
                .await
                .unwrap();
        }
    }

    let manager = web::Data::new(manager);
    let event_rx = web::Data::new(event_rx);
    HttpServer::new(move || {
        App::new()
            .app_data(event_rx.clone())
            .app_data(image_store.clone())
            .app_data(manager.clone())
            .wrap(TracingLogger::default())
            .service(web::scope("api").configure(init_repositories))
            .service(static_files::service())
    })
    .bind(format!("127.0.0.1:{}", CONFIG.server.port))?
    .run()
    .await
}

fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    win_wrapper::path::cd_to_exe();

    let _guard = init_logging();

    #[cfg(windows)]
    win_setup::win_main();

    async_main()
}
