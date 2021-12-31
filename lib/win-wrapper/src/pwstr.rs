use std::{
    ffi::{OsStr, OsString},
    iter,
    os::windows::ffi::OsStrExt,
};
use windows::Win32::Foundation::PWSTR;

pub struct ManagedPwstr(Vec<u16>);

impl ManagedPwstr {
    pub unsafe fn get_pwstr(&mut self) -> PWSTR {
        PWSTR(self.0.as_mut_ptr())
    }
}

impl From<&str> for ManagedPwstr {
    fn from(str: &str) -> Self {
        Self(str.encode_utf16().chain(iter::once(0)).collect())
    }
}

impl From<String> for ManagedPwstr {
    fn from(str: String) -> Self {
        str.as_str().into()
    }
}

impl From<&OsStr> for ManagedPwstr {
    fn from(str: &OsStr) -> Self {
        Self(str.encode_wide().chain(iter::once(0)).collect())
    }
}

impl From<OsString> for ManagedPwstr {
    fn from(str: OsString) -> Self {
        str.as_os_str().into()
    }
}
