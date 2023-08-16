use std::{env, iter, ops::Deref, os::windows::ffi::OsStrExt};
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::ERROR_MORE_DATA,
        System::Registry::{
            RegCloseKey, RegCreateKeyExW, RegGetValueW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
            KEY_CREATE_SUB_KEY, KEY_SET_VALUE, REG_SZ, RRF_RT_REG_SZ,
        },
    },
};

pub use windows::Win32::Foundation::ERROR_ACCESS_DENIED;
use windows::Win32::System::Registry::{RegDeleteValueW, RegOpenKeyExW, REG_OPTION_RESERVED};

struct ManagedHkey(HKEY);
impl ManagedHkey {
    const fn new(key: HKEY) -> Self {
        Self(key)
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
            RegCloseKey(self.0).ok();
        }
    }
}

pub fn add_self_to_autostart(application_name: impl Into<PCWSTR>) -> windows::core::Result<()> {
    unsafe {
        let mut hkey = HKEY(0);
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            0,
            None,
            REG_OPTION_RESERVED,
            KEY_CREATE_SUB_KEY | KEY_SET_VALUE,
            None,
            &mut hkey,
            None,
        )?;
        let hkey = ManagedHkey::new(hkey);

        let path = env::current_exe()
            .unwrap()
            .as_os_str()
            .encode_wide()
            .chain(iter::once(0))
            .collect::<Vec<u16>>();

        RegSetValueExW(
            *hkey,
            application_name.into(),
            0,
            REG_SZ,
            Some(std::slice::from_raw_parts(
                path.as_ptr() as *const u8,
                path.len() * 2,
            )),
        )
    }
}

pub fn check_autostart(application_name: impl Into<PCWSTR>) -> bool {
    unsafe {
        match RegGetValueW(
            HKEY_CURRENT_USER,
            windows::core::w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            application_name.into(),
            RRF_RT_REG_SZ,
            None,
            None,
            None,
        ) {
            Ok(_) => true,
            Err(e) if e.code() == ERROR_MORE_DATA.to_hresult() => true,
            _ => false,
        }
    }
}

pub fn remove_autostart(application_name: impl Into<PCWSTR>) -> windows::core::Result<()> {
    unsafe {
        let mut hkey = HKEY(0);
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        )?;
        let hkey = ManagedHkey::new(hkey);
        RegDeleteValueW(hkey.0, application_name.into())?;

        Ok(())
    }
}
