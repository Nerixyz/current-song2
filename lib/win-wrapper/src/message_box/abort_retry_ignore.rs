use super::MessageBoxOption;
use windows::Win32::UI::WindowsAndMessaging::{
    IDABORT, IDRETRY, MB_ABORTRETRYIGNORE, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE,
};

#[derive(Debug, PartialEq)]
pub enum AbortRetryIgnore {
    Abort,
    Retry,
    Ignore,
}

impl From<MESSAGEBOX_RESULT> for AbortRetryIgnore {
    fn from(value: MESSAGEBOX_RESULT) -> Self {
        match value {
            IDABORT => Self::Abort,
            IDRETRY => Self::Retry,
            _ => Self::Ignore,
        }
    }
}

impl MessageBoxOption for AbortRetryIgnore {
    fn flags() -> MESSAGEBOX_STYLE {
        MB_ABORTRETRYIGNORE
    }
}
