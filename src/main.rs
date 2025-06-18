#![cfg_attr(
    all(feature = "win32-executable", not(test)),
    windows_subsystem = "windows"
)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

mod actors;
mod config;
mod image_store;
mod logging;
mod model;
mod repositories;
mod static_files;
#[cfg(windows)]
mod win_setup;
mod workers;
#[macro_use]
mod macros;
mod utilities;

use config::{ModuleConfig, CONFIG};
use std::sync::Arc;

use crate::{
    actors::manager::{self, Manager},
    image_store::ImageStore,
    logging::init_logging,
    model::ModuleState,
    repositories::init_repositories,
    workers::file_output::output_to_file,
};
use actix::{Actor, Addr};
use actix_web::{web, App, HttpServer};
use std::sync::RwLock;
use tokio::sync::watch;
use tracing_actix_web::TracingLogger;

fn init_channels() -> (watch::Receiver<manager::Event>, Addr<Manager>) {
    let (event_tx, event_rx) = watch::channel(Arc::new(ModuleState::Paused));

    let manager = Manager::new(event_tx).start();

    (event_rx, manager)
}

fn init_common_actors(
    modules: &'static ModuleConfig,
    event_rx: &watch::Receiver<Arc<ModuleState>>,
) {
    if modules.file.enabled {
        let event_rx = event_rx.clone();
        tokio::spawn(async move {
            output_to_file(&modules.file.path, &modules.file.format, event_rx).await;
        });
    }
}

#[cfg(windows)]
async fn init_windows_actors(
    modules: &'static ModuleConfig,
    manager: Addr<Manager>,
    image_store: Arc<RwLock<ImageStore>>,
) {
    if modules.gsmtc.enabled {
        workers::gsmtc::start_spawning(manager, image_store)
            .await
            .unwrap();
    }
}

#[cfg(unix)]
async fn init_unix_actors(
    modules: &'static ModuleConfig,
    manager: Addr<Manager>,
    image_store: Arc<RwLock<ImageStore>>,
) {
    if modules.dbus.enabled {
        workers::dbus::start_spawning(manager, image_store)
            .await
            .unwrap();
    }
}

#[actix_web::main]
async fn async_main() -> std::io::Result<()> {
    let (event_rx, manager) = init_channels();

    let image_store = Arc::new(RwLock::new(ImageStore::new()));

    init_common_actors(&CONFIG.modules, &event_rx);

    #[cfg(windows)]
    init_windows_actors(&CONFIG.modules, manager.clone(), image_store.clone()).await;

    #[cfg(unix)]
    init_unix_actors(&CONFIG.modules, manager.clone(), image_store.clone()).await;

    let image_store: web::Data<_> = image_store.into();
    let manager = web::Data::new(manager);
    let event_rx = web::Data::new(event_rx);
    let srv = HttpServer::new(move || {
        App::new()
            .app_data(event_rx.clone())
            .app_data(image_store.clone())
            .app_data(manager.clone())
            .wrap(TracingLogger::default())
            .service(web::scope("api").configure(init_repositories))
            .service(static_files::theme_css)
            .service(static_files::user_js)
            .service(static_files::service())
    })
    .workers(2);

    match &CONFIG.server.bind {
        config::BindConfig::Single { port } => {
            tracing::info!("Binding on 127.0.0.1:{port}");
            srv.bind((std::net::Ipv4Addr::LOCALHOST, *port))
        }
        config::BindConfig::Multiple { bind } => {
            tracing::info!("Binding on {bind:?}");
            srv.bind(&bind[..])
        }
    }?
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
