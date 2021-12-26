mod actors;
mod config;
mod image_store;
mod model;
mod repositories;
mod workers;

use config::CONFIG;

use crate::{
    actors::{broadcaster, broadcaster::Broadcaster, manager::Manager},
    image_store::ImageStore,
    repositories::init_repositories,
};
use actix::Actor;
use actix_web::{web, App, HttpServer};
use tokio::sync::{watch, RwLock};
use tracing_actix_web::TracingLogger;
use tracing_log::LogTracer;
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter, FmtSubscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    lazy_static::initialize(&CONFIG);
    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();
    //LogTracer::init().unwrap();

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
            .service(
                actix_files::Files::new("/", "js/packages/client/dist").index_file("index.html"),
            )
    })
    .bind(format!("127.0.0.1:{}", CONFIG.server.port))?
    .run()
    .await
}
