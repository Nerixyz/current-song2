use crate::pwstr::ManagedPwstr;
use std::env;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::CloseHandle,
        System::Threading::WaitForSingleObject,
        UI::{
            Shell::{
                ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
            },
            WindowsAndMessaging::SW_NORMAL,
        },
    },
};

pub fn elevate_self() -> windows::core::Result<()> {
    unsafe {
        let verb: ManagedPwstr = "runas".into();
        let file: ManagedPwstr = env::current_exe().unwrap().as_os_str().into();
        let parameters: ManagedPwstr = "--elevated".into();
        let mut info = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            lpVerb: verb.get_pcwstr(),
            lpFile: file.get_pcwstr(),
            lpParameters: parameters.get_pcwstr(),
            lpDirectory: PCWSTR(std::ptr::null()),
            nShow: SW_NORMAL.0 as _,
            fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
            ..Default::default()
        };
        ShellExecuteExW(&mut info)?;
        WaitForSingleObject(info.hProcess, u32::MAX);
        let _ = CloseHandle(info.hProcess);

        Ok(())
    }
}
