use std::{env, iter, ops::Deref, os::windows::ffi::OsStrExt, ptr};
use windows::Win32::{
    Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS, WIN32_ERROR},
    System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegGetValueW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
        KEY_CREATE_SUB_KEY, KEY_SET_VALUE, REG_SZ, RRF_RT_REG_SZ,
    },
};

pub use windows::Win32::Foundation::ERROR_ACCESS_DENIED;
use windows::Win32::System::Registry::{RegDeleteValueW, RegOpenKeyExW};

struct ManagedHkey(HKEY);
impl ManagedHkey {
    fn try_new(ret: WIN32_ERROR, key: HKEY) -> Result<Self, WIN32_ERROR> {
        match ret {
            ERROR_SUCCESS => Ok(Self(key)),
            x => Err(x),
        }
    }
}

impl Deref for ManagedHkey {
    type Target = HKEY;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for ManagedHkey {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}

pub fn add_self_to_autostart(application_name: &str) -> Result<(), WIN32_ERROR> {
    unsafe {
        let mut hkey = 0;
        let hkey = ManagedHkey::try_new(
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                0,
                None,
                0,
                KEY_CREATE_SUB_KEY | KEY_SET_VALUE,
                std::ptr::null(),
                &mut hkey,
                std::ptr::null_mut(),
            ),
            hkey,
        )?;

        let path = env::current_exe()
            .unwrap()
            .as_os_str()
            .encode_wide()
            .chain(iter::once(0))
            .collect::<Vec<u16>>();

        match RegSetValueExW(
            *hkey,
            application_name,
            0,
            REG_SZ,
            path.as_ptr() as *const _,
            path.len() as u32 * 2,
        ) {
            ERROR_SUCCESS => Ok(()),
            x => Err(x),
        }
    }
}

pub fn check_autostart(application_name: &str) -> bool {
    unsafe {
        matches!(
            RegGetValueW(
                HKEY_CURRENT_USER,
                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                application_name,
                RRF_RT_REG_SZ,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut()
            ),
            ERROR_SUCCESS | ERROR_MORE_DATA
        )
    }
}

pub fn remove_autostart(application_name: &str) {
    unsafe {
        let mut hkey = 0;
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            0,
            KEY_SET_VALUE,
            &mut hkey,
        ) != ERROR_SUCCESS
        {
            return;
        }
        RegDeleteValueW(hkey, application_name);
        RegCloseKey(hkey);
    }
}
