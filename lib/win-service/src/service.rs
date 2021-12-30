use crate::alloc::SafePWSTR;
use std::{
    ffi,
    fmt::{Debug, Formatter},
    ops::Deref,
    ptr::NonNull,
};
use windows::Win32::{
    Foundation::{GetLastError, ERROR_INSUFFICIENT_BUFFER, NO_ERROR, PWSTR, WIN32_ERROR},
    Security::SC_HANDLE,
    System::{
        Memory::{LocalAlloc, LocalFree, LMEM_FIXED},
        Services::{
            ChangeServiceConfigW, CloseServiceHandle, QueryServiceConfigW, QueryServiceStatus,
            RegisterServiceCtrlHandlerExW, SetServiceStatus, StartServiceCtrlDispatcherW,
            StartServiceW, QUERY_SERVICE_CONFIGW, SERVICE_ACCEPT_SHUTDOWN, SERVICE_ACCEPT_STOP,
            SERVICE_RUNNING, SERVICE_START_PENDING, SERVICE_STATUS, SERVICE_STATUS_HANDLE,
            SERVICE_STOPPED, SERVICE_TABLE_ENTRYW, SERVICE_WIN32_OWN_PROCESS,
        },
    },
};

pub struct SafeScHandle(pub SC_HANDLE);

impl SafeScHandle {
    pub fn new(handle: SC_HANDLE) -> Option<Self> {
        if handle == 0 {
            None
        } else {
            Some(Self(handle))
        }
    }
}

impl Drop for SafeScHandle {
    fn drop(&mut self) {
        unsafe {
            CloseServiceHandle(self.0);
        }
    }
}

impl Deref for SafeScHandle {
    type Target = SC_HANDLE;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<SC_HANDLE> for SafeScHandle {
    type Error = ();

    fn try_from(value: SC_HANDLE) -> Result<Self, Self::Error> {
        SafeScHandle::new(value).ok_or_else(|| ())
    }
}

#[repr(transparent)]
pub struct QueryServiceConfigWrapper(NonNull<QUERY_SERVICE_CONFIGW>);

impl QueryServiceConfigWrapper {
    /// this doesn't change the internal config!
    pub fn set_bin_path(&self, sc_handle: SC_HANDLE, bin_path: &str) -> Result<(), WIN32_ERROR> {
        let pw_bin_path = SafePWSTR::alloc(bin_path);
        unsafe {
            ChangeServiceConfigW(
                sc_handle,
                self.dwServiceType,
                self.dwStartType,
                self.dwErrorControl,
                *pw_bin_path,
                None,
                std::ptr::null_mut(),
                None,
                None,
                None,
                None,
            )
            .as_bool()
            .then(|| ())
            .ok_or_else(|| GetLastError())
        }
    }
}

impl Deref for QueryServiceConfigWrapper {
    type Target = QUERY_SERVICE_CONFIGW;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl Drop for QueryServiceConfigWrapper {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0.as_ptr() as isize);
        }
    }
}

pub fn query_service_config(
    service_handle: SC_HANDLE,
) -> Result<QueryServiceConfigWrapper, WIN32_ERROR> {
    let mut desired_size = 0;
    unsafe {
        QueryServiceConfigW(service_handle, std::ptr::null_mut(), 0, &mut desired_size);
        match GetLastError() {
            ERROR_INSUFFICIENT_BUFFER => (),
            x => return Err(x),
        }
        let service_config: *mut QUERY_SERVICE_CONFIGW =
            LocalAlloc(LMEM_FIXED, desired_size as usize) as usize as _;
        if !QueryServiceConfigW(
            service_handle,
            service_config,
            desired_size,
            &mut desired_size,
        )
        .as_bool()
            || service_config.is_null()
        {
            LocalFree(service_config as isize);
            Err(GetLastError())
        } else {
            // we checked for null in the if condition
            Ok(QueryServiceConfigWrapper(NonNull::new_unchecked(
                service_config,
            )))
        }
    }
}

pub fn start_service(service_handle: SC_HANDLE) -> Result<(), WIN32_ERROR> {
    unsafe {
        if StartServiceW(service_handle, 0, std::ptr::null()).as_bool() {
            Ok(())
        } else {
            Err(GetLastError())
        }
    }
}

#[repr(transparent)]
pub struct ServiceStatusWrapper(SERVICE_STATUS);

impl From<SERVICE_STATUS> for ServiceStatusWrapper {
    fn from(s: SERVICE_STATUS) -> Self {
        Self(s)
    }
}

impl Deref for ServiceStatusWrapper {
    type Target = SERVICE_STATUS;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for ServiceStatusWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceStatus")
            .field("dwServiceType", &self.dwServiceType)
            .field("dwCurrentState", &self.dwCurrentState)
            .field("dwControlsAccepted", &self.dwControlsAccepted)
            .field("dwWin32ExitCode", &self.dwWin32ExitCode)
            .field("dwServiceSpecificExitCode", &self.dwServiceSpecificExitCode)
            .field("dwCheckPoint", &self.dwCheckPoint)
            .field("dwWaitHint", &self.dwWaitHint)
            .finish()
    }
}

pub fn query_service_status(
    service_handle: SC_HANDLE,
) -> Result<ServiceStatusWrapper, WIN32_ERROR> {
    unsafe {
        let mut status = SERVICE_STATUS::default();
        if QueryServiceStatus(service_handle, &mut status).as_bool() {
            Ok(status.into())
        } else {
            Err(GetLastError())
        }
    }
}

pub type ServiceMain =
    unsafe extern "system" fn(dw_num_services_args: u32, lp_service_arg_vectors: *mut PWSTR);

pub fn start_service_control_dispatcher(
    service_name: &str,
    main: ServiceMain,
) -> Result<(), WIN32_ERROR> {
    let name_pwstr = SafePWSTR::alloc(service_name);
    let table = [
        SERVICE_TABLE_ENTRYW {
            lpServiceProc: Some(main),
            lpServiceName: *name_pwstr,
        },
        SERVICE_TABLE_ENTRYW::default(),
    ];
    unsafe {
        if StartServiceCtrlDispatcherW(table.as_ptr()).as_bool() {
            Ok(())
        } else {
            Err(GetLastError())
        }
    }
}

pub type ServiceCtrlHandler = unsafe extern "system" fn(
    dw_control: u32,
    dw_event_type: u32,
    lp_event_data: *mut ffi::c_void,
    lp_context: *mut ffi::c_void,
) -> u32;
pub type ServiceStatusHandle = SERVICE_STATUS_HANDLE;

pub fn register_service_ctrl_handler(
    service_name: &str,
    handler: ServiceCtrlHandler,
    data: *mut ffi::c_void,
) -> Result<ServiceStatusHandle, WIN32_ERROR> {
    let name_pwstr = SafePWSTR::alloc(service_name);

    unsafe {
        match RegisterServiceCtrlHandlerExW(*name_pwstr, Some(handler), data) {
            0 => Err(GetLastError()),
            handle => Ok(handle),
        }
    }
}

pub struct ServiceState {
    status_handle: ServiceStatusHandle,
    status: SERVICE_STATUS,
}

impl ServiceState {
    pub fn new(status_handle: ServiceStatusHandle) -> Self {
        Self {
            status_handle,
            status: SERVICE_STATUS {
                dwServiceType: SERVICE_WIN32_OWN_PROCESS,
                dwCurrentState: SERVICE_START_PENDING,
                dwControlsAccepted: SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN,
                dwWin32ExitCode: NO_ERROR,
                dwServiceSpecificExitCode: 0,
                dwCheckPoint: 0,
                dwWaitHint: 1000,
            },
        }
    }

    fn signal(&self) {
        unsafe {
            SetServiceStatus(self.status_handle, &self.status);
        }
    }

    pub fn signal_stop(&mut self, exit_code: u32) {
        self.status.dwCurrentState = SERVICE_STOPPED;
        self.status.dwWin32ExitCode = exit_code;
        self.signal();
    }

    pub fn signal_started(&mut self) {
        self.status.dwCurrentState = SERVICE_RUNNING;
        self.signal();
    }
}
