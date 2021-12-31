use super::MessageBoxOption;
use windows::Win32::UI::WindowsAndMessaging::{
    IDYES, MB_YESNO, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE,
};

#[derive(Debug, PartialEq)]
pub enum YesNo {
    Yes,
    No,
}

impl From<MESSAGEBOX_RESULT> for YesNo {
    fn from(value: MESSAGEBOX_RESULT) -> Self {
        match value {
            IDYES => Self::Yes,
            _ => Self::No,
        }
    }
}

impl MessageBoxOption for YesNo {
    fn flags() -> MESSAGEBOX_STYLE {
        MB_YESNO
    }
}
