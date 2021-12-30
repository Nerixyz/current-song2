use actix_web::{rt::System, web, App, HttpServer, Responder};
use std::sync::mpsc;
use win_service::{
    check_and_install_service, control_handler, message_box::message_box,
    start_service_control_dispatcher, CheckAndInstall, PWSTR,
};

async fn index() -> impl Responder {
    "WOOOW"
}

async fn async_main(shutdown_signal: Option<mpsc::Receiver<()>>) -> std::io::Result<()> {
    let srv = HttpServer::new(|| App::new().default_service(web::route().to(index)))
        .bind("127.0.0.1:32094")?
        .run();

    if let Some(shutdown_signal) = shutdown_signal {
        let handle = srv.handle();
        std::thread::spawn(move || {
            if shutdown_signal.recv().is_ok() {
                futures::executor::block_on(handle.stop(true));
            }
        });
    }
    srv.await
}

fn actual_main(shutdown_signal: Option<mpsc::Receiver<()>>) -> std::io::Result<()> {
    System::new().block_on(async move { async_main(shutdown_signal).await })
}

unsafe extern "system" fn service_main(_: u32, _: *mut PWSTR) {
    control_handler::run_service(SERVICE_NAME, |stop_signal| {
        match actual_main(Some(stop_signal)) {
            Ok(_) => 0,
            Err(e) => e.raw_os_error().unwrap_or(1),
        }
    })
}

const SERVICE_NAME: &str = "alienator";

#[derive(Default)]
struct BinArgs {
    from_service: bool,
    skip_check: bool,
    force_check: bool,
}

fn parse_args() -> BinArgs {
    let mut args = BinArgs::default();
    for arg in std::env::args() {
        match arg.as_str() {
            "--from-service" => args.from_service = true,
            "--skip-check" => args.skip_check = true,
            // also skip if we're running in elevated mode
            // there, we already checked the service
            "--force-check" | "--with-admin" => args.force_check = true,
            _ => (),
        }
    }
    args
}

fn main() {
    let args = parse_args();
    if args.from_service {
        start_service_control_dispatcher(SERVICE_NAME, service_main).unwrap();
        return;
    } else if args.skip_check {
        actual_main(None).unwrap();
    } else {
        match check_and_install_service(SERVICE_NAME, args.force_check) {
            CheckAndInstall::AlreadyStarted =>
                message_box("The service is already started, use --skip-check to start the server regardless, and --force-check to check the service's path!"),
            CheckAndInstall::Created(x) => message_box(&format!("The service has been created/updated ({:?}).", x)),
            CheckAndInstall::FailedCreating(x) => message_box(&format!("The service couldn't be created/updated: {:?}.\nReport this on the GitHub repository!", x)),
        }
    }
}
