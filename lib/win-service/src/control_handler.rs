use crate::{register_service_ctrl_handler, ServiceState};
use std::{ffi, sync::mpsc};
use tracing::{event, Level};
use windows::Win32::{
    Foundation::{ERROR_CALL_NOT_IMPLEMENTED, NO_ERROR},
    System::Services::{
        SERVICE_CONTROL_CONTINUE, SERVICE_CONTROL_INTERROGATE, SERVICE_CONTROL_SHUTDOWN,
        SERVICE_CONTROL_STOP,
    },
};

struct ServiceCtx {
    stop_signal: mpsc::Sender<()>,
}

unsafe extern "system" fn service_ctrl_handler(
    dw_control: u32,
    _dw_event_type: u32,
    _lp_event_data: *mut ffi::c_void,
    lp_context: *mut ffi::c_void,
) -> u32 {
    match dw_control {
        SERVICE_CONTROL_INTERROGATE | SERVICE_CONTROL_CONTINUE => NO_ERROR,
        SERVICE_CONTROL_SHUTDOWN | SERVICE_CONTROL_STOP => {
            let ctx: *mut ServiceCtx = lp_context as _;
            (*ctx).stop_signal.send(()).ok();
            NO_ERROR
        }
        _ => ERROR_CALL_NOT_IMPLEMENTED,
    }
}

fn register_sane_ctrl_handler(service_name: &str) -> (ServiceState, mpsc::Receiver<()>) {
    let (tx, rx) = mpsc::channel();
    let ctrl_ctx = Box::leak(Box::new(ServiceCtx { stop_signal: tx }));
    let handle = register_service_ctrl_handler(
        service_name,
        service_ctrl_handler,
        ctrl_ctx as *mut ServiceCtx as _,
    )
    .unwrap();
    (ServiceState::new(handle), rx)
}

pub fn run_service<F>(service_name: &str, main_fn: F) -> !
where
    F: FnOnce(mpsc::Receiver<()>) -> i32,
{
    let (mut state, stop_signal) = register_sane_ctrl_handler(service_name);

    state.signal_started();
    let res = main_fn(stop_signal);
    state.signal_stop(unsafe { std::mem::transmute(res) });

    event!(Level::INFO, "Stopping");
    std::process::exit(res)
}
