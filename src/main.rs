mod actors;
mod config;
mod image_store;
mod model;
mod repositories;
mod static_files;
#[cfg(windows)]
mod win_svc;
mod workers;

use config::CONFIG;
use std::sync::mpsc;

use crate::{
    actors::{broadcaster, broadcaster::Broadcaster, manager::Manager},
    image_store::ImageStore,
    repositories::init_repositories,
};
use actix::Actor;
use actix_web::{rt::System, web, App, HttpServer};
use std::io;
use tokio::sync::{watch, RwLock};
use tracing::{event, Level};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

/// This is where the actual logic of the app lives.
/// **`unwrap()` cannot be called in here!**
async fn async_main(stop_signal: Option<mpsc::Receiver<()>>) -> io::Result<()> {
    let (event_tx, event_rx) =
        watch::channel::<broadcaster::Event>(serde_json::json!({"type": "Paused"}).to_string());

    let broadcaster = Broadcaster::new(event_tx).start();
    let manager = Manager::new(broadcaster.recipient()).start();

    let image_store = web::Data::new(RwLock::new(ImageStore::new()));

    if cfg!(windows) {
        if CONFIG.modules.gsmtc.enabled {
            workers::gsmtc::start_spawning(manager.clone(), image_store.clone().into_inner())
                .await
                .map_err(|e| {
                    event!(Level::ERROR, error = %e, "GSMTC couldn't start spawning");
                    io::Error::from_raw_os_error(2)
                })?;
        }
    }

    let manager = web::Data::new(manager);
    let event_rx = web::Data::new(event_rx);
    let srv = HttpServer::new(move || {
        App::new()
            .app_data(event_rx.clone())
            .app_data(image_store.clone())
            .app_data(manager.clone())
            .wrap(TracingLogger::default())
            .service(web::scope("api").configure(init_repositories))
            .service(static_files::service())
    })
    .bind(format!("127.0.0.1:{}", CONFIG.server.port))?
    .run();

    if let Some(shutdown_signal) = stop_signal {
        let handle = srv.handle();
        std::thread::spawn(move || {
            if shutdown_signal.recv().is_ok() {
                event!(Level::INFO, "Received stop");
                futures::executor::block_on(handle.stop(false));
                event!(Level::INFO, "stopped");
            }
        });
    }

    srv.await
}

/// This is basically the macro-expansion of #\[actix_web::main]
/// It's explicit here, to distinguish between async and sync.
fn actual_main(shutdown_signal: Option<mpsc::Receiver<()>>) -> std::io::Result<()> {
    let res = System::new().block_on(async move { async_main(shutdown_signal).await });
    event!(Level::INFO, "after system");
    res
}

#[cfg(windows)]
unsafe extern "system" fn service_main(_: u32, _: *mut win_service::PWSTR) {
    win_service::control_handler::run_service(
        win_svc::SERVICE_NAME,
        |stop_signal| match actual_main(Some(stop_signal)) {
            Ok(_) => 0,
            Err(e) => e.raw_os_error().unwrap_or(1),
        },
    )
}

fn main() {
    #[cfg(windows)]
    win_service::path::cd_to_exe();

    lazy_static::initialize(&CONFIG);
    let file_appender = tracing_appender::rolling::never("", "current_song2.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let collector = tracing_subscriber::registry()
        .with(
            EnvFilter::from_default_env()
                .add_directive(Level::TRACE.into())
                .add_directive("tokio=debug".parse().unwrap())
                .add_directive("runtime=debug".parse().unwrap()),
        )
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_ansi(false).with_writer(non_blocking));
    tracing::subscriber::set_global_default(collector).unwrap();

    #[cfg(windows)]
    win_svc::win_main(actual_main, service_main);
    #[cfg(not(windows))]
    actual_main(None).unwrap();
}
