use crate::pwstr::ManagedPwstr;
use std::env;
use windows::Win32::{
    Foundation::{CloseHandle, GetLastError, PWSTR, WIN32_ERROR},
    System::Threading::WaitForSingleObject,
    UI::{
        Shell::{ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW},
        WindowsAndMessaging::SW_NORMAL,
    },
};

pub fn elevate_self() -> Result<(), WIN32_ERROR> {
    unsafe {
        let mut verb: ManagedPwstr = "runas".into();
        let mut file: ManagedPwstr = env::current_exe().unwrap().as_os_str().into();
        let mut parameters: ManagedPwstr = "--elevated".into();
        let mut info = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            lpVerb: verb.get_pwstr(),
            lpFile: file.get_pwstr(),
            lpParameters: parameters.get_pwstr(),
            lpDirectory: PWSTR(std::ptr::null_mut()),
            nShow: SW_NORMAL.0 as _,
            fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
            ..Default::default()
        };
        if !ShellExecuteExW(&mut info).as_bool() {
            return Err(GetLastError());
        }
        WaitForSingleObject(info.hProcess, u32::MAX);
        CloseHandle(info.hProcess);

        Ok(())
    }
}
