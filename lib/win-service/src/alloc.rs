use std::{iter, ops::Deref};
use windows::Win32::Foundation::PWSTR;

pub struct SafePWSTR(PWSTR);

impl SafePWSTR {
    pub fn alloc(str: &str) -> Self {
        Self(alloc_pwstr(str))
    }
}

impl Drop for SafePWSTR {
    fn drop(&mut self) {
        unsafe {
            dealloc_pwstr(self.0);
        }
    }
}

impl AsRef<PWSTR> for SafePWSTR {
    fn as_ref(&self) -> &PWSTR {
        &self.0
    }
}

impl Deref for SafePWSTR {
    type Target = PWSTR;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn pwstr_to_string(str: PWSTR) -> Option<String> {
    if !str.is_null() {
        unsafe {
            let len = (0..).take_while(|&i| *str.0.offset(i) != 0).count();
            let slice = std::slice::from_raw_parts(str.0, len);
            Some(String::from_utf16_lossy(slice))
        }
    } else {
        None
    }
}

fn alloc_pwstr(str: &str) -> PWSTR {
    PWSTR(Box::<[u16]>::into_raw(
        str.encode_utf16()
            .chain(iter::once(0))
            .collect::<Vec<u16>>()
            .into_boxed_slice(),
    ) as _)
}

unsafe fn dealloc_pwstr(str: PWSTR) {
    if !str.is_null() {
        Box::from_raw(str.0);
    }
}
