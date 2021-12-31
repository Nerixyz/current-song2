use super::MessageBoxOption;
use windows::Win32::UI::WindowsAndMessaging::{MB_OK, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE};

#[derive(Debug, Eq, PartialEq)]
pub struct Okay;

impl From<MESSAGEBOX_RESULT> for Okay {
    fn from(_: MESSAGEBOX_RESULT) -> Self {
        Self
    }
}

impl MessageBoxOption for Okay {
    fn flags() -> MESSAGEBOX_STYLE {
        MB_OK
    }
}
