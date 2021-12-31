use super::MessageBoxOption;
use windows::Win32::UI::WindowsAndMessaging::{
    IDRETRY, MB_RETRYCANCEL, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE,
};

#[derive(Debug, PartialEq)]
pub enum RetryCancel {
    Retry,
    Cancel,
}

impl From<MESSAGEBOX_RESULT> for RetryCancel {
    fn from(value: MESSAGEBOX_RESULT) -> Self {
        match value {
            IDRETRY => Self::Retry,
            _ => Self::Cancel,
        }
    }
}

impl MessageBoxOption for RetryCancel {
    fn flags() -> MESSAGEBOX_STYLE {
        MB_RETRYCANCEL
    }
}
