pub mod alloc;
pub mod control_handler;
pub mod message_box;
pub mod path;
mod service;
mod shell;

pub use service::{
    register_service_ctrl_handler, start_service_control_dispatcher, ServiceMain, ServiceState,
};
pub use windows::Win32::Foundation::PWSTR;

use crate::{
    alloc::{pwstr_to_string, SafePWSTR},
    service::{query_service_config, query_service_status, start_service, SafeScHandle},
    shell::run_self_as_admin,
};
use std::{env, ffi, process::Command};
use tracing::{event, Level};
use windows::Win32::{
    Foundation::{GetLastError, ERROR_ACCESS_DENIED, ERROR_SUCCESS, WIN32_ERROR},
    System::{
        Registry::{RegOpenKeyExW, RegSetKeyValueW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_DWORD},
        Services::{
            CreateServiceW, OpenSCManagerW, OpenServiceW, SC_MANAGER_ALL_ACCESS,
            SC_MANAGER_CONNECT, SERVICE_ALL_ACCESS, SERVICE_AUTO_START, SERVICE_CHANGE_CONFIG,
            SERVICE_ERROR_NORMAL, SERVICE_QUERY_CONFIG, SERVICE_QUERY_STATUS, SERVICE_RUNNING,
            SERVICE_START, SERVICE_WIN32_OWN_PROCESS,
        },
    },
};

pub mod control_codes {
    pub use windows::Win32::{
        Foundation::{ERROR_CALL_NOT_IMPLEMENTED, NO_ERROR},
        System::Services::{
            SERVICE_CONTROL_CONTINUE, SERVICE_CONTROL_DEVICEEVENT,
            SERVICE_CONTROL_HARDWAREPROFILECHANGE, SERVICE_CONTROL_INTERROGATE,
            SERVICE_CONTROL_LOWRESOURCES, SERVICE_CONTROL_NETBINDADD,
            SERVICE_CONTROL_NETBINDDISABLE, SERVICE_CONTROL_NETBINDENABLE,
            SERVICE_CONTROL_NETBINDREMOVE, SERVICE_CONTROL_PARAMCHANGE, SERVICE_CONTROL_PAUSE,
            SERVICE_CONTROL_POWEREVENT, SERVICE_CONTROL_PRESHUTDOWN, SERVICE_CONTROL_SESSIONCHANGE,
            SERVICE_CONTROL_SHUTDOWN, SERVICE_CONTROL_STATUS_REASON_INFO, SERVICE_CONTROL_STOP,
        },
    };
}

fn service_reg_key(application_name: &str) -> String {
    format!("SOFTWARE\\{}\\service_registered", application_name)
}

fn check_key(application_name: &str) -> bool {
    let mut key = HKEY::default();
    unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            service_reg_key(application_name),
            0,
            KEY_READ,
            &mut key,
        ) == ERROR_SUCCESS
    }
}

fn set_key(application_name: &str) -> Result<(), WIN32_ERROR> {
    unsafe {
        let value1 = 1u32;
        match RegSetKeyValueW(
            HKEY_LOCAL_MACHINE,
            service_reg_key(application_name),
            None,
            REG_DWORD,
            &value1 as *const u32 as *const ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        ) {
            ERROR_SUCCESS => Ok(()),
            e => Err(e),
        }
    }
}

fn check_key_and_service(application_name: &str) -> bool {
    unsafe {
        if !check_key(application_name) {
            return false;
        }

        let handle: SafeScHandle = match OpenSCManagerW(None, None, SC_MANAGER_CONNECT)
            .try_into()
            .map_err(|_| GetLastError())
        {
            Ok(handle) => handle,
            Err(_) => return false,
        };
        let pw_app_name = SafePWSTR::alloc(application_name);
        let svc: SafeScHandle = match OpenServiceW(*handle, *pw_app_name, SERVICE_QUERY_STATUS)
            .try_into()
            .map_err(|_| GetLastError())
        {
            Ok(svc) => svc,
            _ => return false,
        };

        query_service_status(*svc)
            .map(|status| status.dwCurrentState == SERVICE_RUNNING)
            .unwrap_or(false)
    }
}

fn make_start_command() -> String {
    format!(
        "{:?}",
        Command::new(env::current_exe().unwrap()).arg("--from-service")
    )
}

#[derive(Debug)]
pub enum HandleExistingError {
    NoServiceConfig(WIN32_ERROR),
    NoBinaryPath,
    NoServiceStatus(WIN32_ERROR),
    CannotSetBinaryPath(WIN32_ERROR),
    CannotStartService(WIN32_ERROR),
    CannotSetRegistryKey(WIN32_ERROR),
}

fn handle_existing_service(
    application_name: &str,
    prev_service: SafeScHandle,
) -> Result<(), HandleExistingError> {
    let config = match query_service_config(*prev_service) {
        Ok(cfg) => cfg,
        Err(e) => return Err(HandleExistingError::NoServiceConfig(e)),
    };
    let binpath =
        pwstr_to_string(config.lpBinaryPathName).ok_or(HandleExistingError::NoBinaryPath)?;
    let expected_bin_path = make_start_command();
    if binpath != expected_bin_path {
        event!(Level::INFO, %binpath, %expected_bin_path, "Unexpected binary path - changing");
        config
            .set_bin_path(*prev_service, &expected_bin_path)
            .map_err(HandleExistingError::CannotSetBinaryPath)?;
    }

    if query_service_status(*prev_service)
        .map_err(HandleExistingError::NoServiceStatus)?
        .dwCurrentState
        != SERVICE_RUNNING
    {
        event!(Level::INFO, "Service is not running, starting...");
        start_service(*prev_service).map_err(HandleExistingError::CannotStartService)?;
    }

    event!(Level::INFO, "Service should be up");
    set_key(application_name).map_err(HandleExistingError::CannotSetRegistryKey)
}

#[derive(Debug)]
pub enum NewServiceError {
    OpenScManagerError(WIN32_ERROR),
    CreateError(WIN32_ERROR),
    RunAdminError(WIN32_ERROR),
    RunAdminNonZeroExit(u32),
    StartError(WIN32_ERROR),
    HandleExistingError(HandleExistingError),
    CannotSetRegistryKey(WIN32_ERROR),
}

#[derive(Debug)]
pub enum NewServiceAction {
    RanAsAdmin,
    HandledExisting,
    CreatedAndStartedService,
}

fn create_this_thing(application_name: &str) -> Result<NewServiceAction, NewServiceError> {
    unsafe {
        let sc_handle: SafeScHandle =
            match OpenSCManagerW(None, None, SC_MANAGER_ALL_ACCESS).try_into() {
                Ok(handle) => handle,
                Err(_) => {
                    return match GetLastError() {
                        ERROR_ACCESS_DENIED => match run_self_as_admin() {
                            Ok(0) => Ok(NewServiceAction::RanAsAdmin),
                            Ok(non_zero_exit) => {
                                Err(NewServiceError::RunAdminNonZeroExit(non_zero_exit))
                            }
                            Err(e) => Err(NewServiceError::RunAdminError(e)),
                        },
                        e => Err(NewServiceError::OpenScManagerError(e)),
                    }
                }
            };

        if let Ok(prev_service) = OpenServiceW(
            *sc_handle,
            application_name,
            SERVICE_QUERY_CONFIG | SERVICE_CHANGE_CONFIG | SERVICE_QUERY_STATUS | SERVICE_START,
        )
        .try_into()
        {
            event!(Level::INFO, "Service exists, attempting to patch");
            return handle_existing_service(application_name, prev_service)
                .map(|_| NewServiceAction::HandledExisting)
                .map_err(NewServiceError::HandleExistingError);
        }

        let new_service: SafeScHandle = CreateServiceW(
            *sc_handle,
            application_name,
            application_name,
            SERVICE_ALL_ACCESS,
            SERVICE_WIN32_OWN_PROCESS,
            SERVICE_AUTO_START,
            SERVICE_ERROR_NORMAL,
            make_start_command(),
            None,
            std::ptr::null_mut(),
            None,
            None,
            None,
        )
        .try_into()
        .map_err(|_| NewServiceError::CreateError(GetLastError()))?;

        set_key(application_name).map_err(NewServiceError::CannotSetRegistryKey)?;
        start_service(*new_service).map_err(NewServiceError::StartError)?;
        event!(Level::INFO, "Service started");

        Ok(NewServiceAction::CreatedAndStartedService)
    }
}

#[derive(Debug)]
pub enum CheckAndInstall {
    AlreadyStarted,
    Created(NewServiceAction),
    FailedCreating(NewServiceError),
}

pub fn check_and_install_service(application_name: &str, force_check: bool) -> CheckAndInstall {
    event!(Level::INFO, "Checking for service {}", application_name);
    if !force_check && check_key_and_service(application_name) {
        event!(Level::INFO, "Service is already running");
        CheckAndInstall::AlreadyStarted
    } else {
        match create_this_thing(application_name) {
            Ok(created) => CheckAndInstall::Created(created),
            Err(failed) => {
                event!(Level::ERROR, error = ?failed, "Service creation/patching failed");
                CheckAndInstall::FailedCreating(failed)
            }
        }
    }
}
