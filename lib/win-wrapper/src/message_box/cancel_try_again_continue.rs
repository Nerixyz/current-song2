use super::MessageBoxOption;
use windows::Win32::UI::WindowsAndMessaging::{
    IDCONTINUE, IDTRYAGAIN, MB_CANCELTRYCONTINUE, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE,
};

#[derive(Debug, PartialEq, Eq)]
pub enum CancelTryAgainContinue {
    Cancel,
    TryAgain,
    Continue,
}

impl From<MESSAGEBOX_RESULT> for CancelTryAgainContinue {
    fn from(value: MESSAGEBOX_RESULT) -> Self {
        match value {
            IDTRYAGAIN => Self::TryAgain,
            IDCONTINUE => Self::Continue,
            _ => Self::Cancel,
        }
    }
}

impl MessageBoxOption for CancelTryAgainContinue {
    fn flags() -> MESSAGEBOX_STYLE {
        MB_CANCELTRYCONTINUE
    }
}
