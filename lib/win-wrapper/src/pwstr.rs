use std::{
    ffi::{OsStr, OsString},
    iter,
    os::windows::ffi::OsStrExt,
};
use windows::core::PCWSTR;

pub struct ManagedPwstr(Vec<u16>);

impl ManagedPwstr {
    pub unsafe fn get_pcwstr(&self) -> PCWSTR {
        PCWSTR(self.0.as_ptr())
    }

    pub fn alloc(len: usize) -> Self {
        Self(vec![0; len])
    }

    pub fn as_mut_slice(&mut self) -> &mut [u16] {
        &mut self.0
    }
}

impl PartialEq for ManagedPwstr {
    fn eq(&self, other: &Self) -> bool {
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            if *a != *b {
                return false;
            } else if *a == 0 {
                // both values are equal, we hit a null terminator
                // strings must be equal
                return true;
            }
        }
        // we didn't see a null terminator yet
        false
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
