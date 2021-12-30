use std::sync::mpsc;
use win_service::{
    check_and_install_service, message_box::message_box, start_service_control_dispatcher,
    CheckAndInstall, ServiceMain,
};

#[cfg(debug_assertions)]
pub const SERVICE_NAME: &str = "CurrentSong2_DEV";
#[cfg(not(debug_assertions))]
pub const SERVICE_NAME: &str = "CurrentSong2";

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

pub fn win_main<F>(actual_main: F, service_main: ServiceMain)
where
    F: FnOnce(Option<mpsc::Receiver<()>>) -> std::io::Result<()>,
{
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
