use super::MessageBoxOption;
use windows::Win32::UI::WindowsAndMessaging::{
    IDNO, IDYES, MB_YESNOCANCEL, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE,
};

#[derive(Debug, PartialEq)]
pub enum YesNoCancel {
    Yes,
    No,
    Cancel,
}

impl From<MESSAGEBOX_RESULT> for YesNoCancel {
    fn from(value: MESSAGEBOX_RESULT) -> Self {
        match value {
            IDYES => Self::Yes,
            IDNO => Self::No,
            _ => Self::Cancel,
        }
    }
}

impl MessageBoxOption for YesNoCancel {
    fn flags() -> MESSAGEBOX_STYLE {
        MB_YESNOCANCEL
    }
}
