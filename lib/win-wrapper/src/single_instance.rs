use crate::pwstr::ManagedPwstr;
use std::{env, mem};
use windows::{
    core::{Error, Result, HRESULT, HSTRING},
    Win32::{
        Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, ERROR_NOT_FOUND, MAX_PATH},
        System::{
            ProcessStatus::{EnumProcesses, GetModuleFileNameExW},
            Threading::{
                CreateMutexW, OpenProcess, TerminateProcess, PROCESS_QUERY_INFORMATION,
                PROCESS_TERMINATE,
            },
        },
    },
};

/// Tries to create and lock on a system wide mutex
///
/// If the mutex is already locked, then another application locked the mutex and another instance is already running.
///
/// Returns `false` if another instance is already running, and `true` if we are the only instance running.
pub fn try_create_new_instance(unique_instance_id: &HSTRING) -> bool {
    unsafe {
        match CreateMutexW(None, true, unique_instance_id) {
            Ok(_) => true,
            Err(e) if e.code() == ERROR_ALREADY_EXISTS.to_hresult() => false,
            Err(x) => {
                eprintln!("Unexpected error - {:?}", x);
                false
            }
        }
    }
}

pub fn kill_other_instances_of_this_application() -> Result<()> {
    let mut path_buf = ManagedPwstr::alloc(MAX_PATH as usize + 1);
    let this_path = ManagedPwstr::from(env::current_exe().unwrap().into_os_string());

    let (processes, n_processes) = get_all_processes()?;
    let pid = match processes
        .into_iter()
        .take(n_processes as usize)
        .find(|pid| cmp_path(*pid, &this_path, &mut path_buf))
    {
        Some(pid) => pid,
        None => return Err(Error::from(HRESULT::from(ERROR_NOT_FOUND))),
    };
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, pid)?;

        TerminateProcess(handle, u32::MAX)?;
        let _ = CloseHandle(handle);
    }

    Ok(())
}

fn get_all_processes() -> Result<(Vec<u32>, u32)> {
    let mut buf = vec![0u32; 1024];
    let mut returned_bytes = 0;
    unsafe {
        EnumProcesses(
            buf.as_mut_ptr(),
            (mem::size_of::<u32>() * buf.len()) as u32,
            &mut returned_bytes,
        )?;
    }
    Ok((buf, returned_bytes / (mem::size_of::<u32>() as u32)))
}

fn cmp_path(pid: u32, path: &ManagedPwstr, path_buf: &mut ManagedPwstr) -> bool {
    unsafe {
        let proc = match OpenProcess(PROCESS_QUERY_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return false,
        };
        let chars = GetModuleFileNameExW(proc, None, path_buf.as_mut_slice());
        let _ = CloseHandle(proc);
        if chars == 0 {
            false
        } else {
            path == path_buf
        }
    }
}
