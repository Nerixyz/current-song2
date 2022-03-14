#![windows_subsystem = "windows"]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]

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
mod utilities;

use config::CONFIG;
use std::sync::Arc;

use crate::{
    actors::manager::{self, Manager},
    image_store::ImageStore,
    logging::init_logging,
    model::ModuleState,
    repositories::init_repositories,
};
use actix::{Actor, Addr};
use actix_web::{web, App, HttpServer};
use tokio::sync::{watch, RwLock};
use tracing_actix_web::TracingLogger;

fn init_channels() -> (watch::Receiver<manager::Event>, Addr<Manager>) {
    let (event_tx, event_rx) = watch::channel(Arc::new(ModuleState::Paused));

    let manager = Manager::new(event_tx).start();

    (event_rx, manager)
}

#[cfg(windows)]
async fn init_windows_actors(manager: Addr<Manager>, image_store: Arc<RwLock<ImageStore>>) {
    if CONFIG.modules.gsmtc.enabled {
        workers::gsmtc::start_spawning(manager, image_store)
            .await
            .unwrap();
    }
}

#[actix_web::main]
async fn async_main() -> std::io::Result<()> {
    let (event_rx, manager) = init_channels();

    let image_store = Arc::new(RwLock::new(ImageStore::new()));

    #[cfg(windows)]
    init_windows_actors(manager.clone(), image_store.clone()).await;

    let image_store: web::Data<_> = image_store.into();
    let manager = web::Data::new(manager);
    let event_rx = web::Data::new(event_rx);
    HttpServer::new(move || {
        App::new()
            .app_data(event_rx.clone())
            .app_data(image_store.clone())
            .app_data(manager.clone())
            .wrap(TracingLogger::default())
            .service(web::scope("api").configure(init_repositories))
            .service(static_files::theme_css)
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
