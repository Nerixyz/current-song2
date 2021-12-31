use super::MessageBoxOption;
use windows::Win32::UI::WindowsAndMessaging::{
    IDOK, MB_OKCANCEL, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE,
};

#[derive(Debug, PartialEq)]
pub enum OkayCancel {
    Okay,
    Cancel,
}

impl From<MESSAGEBOX_RESULT> for OkayCancel {
    fn from(value: MESSAGEBOX_RESULT) -> Self {
        match value {
            IDOK => Self::Okay,
            _ => Self::Cancel,
        }
    }
}

impl MessageBoxOption for OkayCancel {
    fn flags() -> MESSAGEBOX_STYLE {
        MB_OKCANCEL
    }
}
