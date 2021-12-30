use crate::alloc::SafePWSTR;
use std::env;
use windows::Win32::{
    Foundation::{GetLastError, PWSTR, WIN32_ERROR},
    System::Threading::{GetExitCodeProcess, WaitForSingleObject},
    UI::Shell::{ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW},
};

pub fn run_self_as_admin() -> Result<u32, WIN32_ERROR> {
    let verb = SafePWSTR::alloc("runas");
    let file = SafePWSTR::alloc(env::current_exe().unwrap().to_string_lossy().as_ref());
    let parameters = SafePWSTR::alloc(if cfg!(not(test)) { "--with-admin" } else { "" });
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        lpVerb: *verb,
        lpFile: *file,
        lpParameters: *parameters,
        lpDirectory: PWSTR(std::ptr::null_mut()),
        nShow: 1, // SW_NORMAL
        ..Default::default()
    };
    unsafe {
        return if ShellExecuteExW(&mut info).as_bool() {
            assert!(info.hInstApp > 32);
            WaitForSingleObject(info.hProcess, u32::MAX);
            let mut exit_code = 0;
            if GetExitCodeProcess(info.hProcess, &mut exit_code).as_bool() {
                Ok(exit_code)
            } else {
                Err(GetLastError())
            }
        } else {
            Err(GetLastError())
        };
    }
}
