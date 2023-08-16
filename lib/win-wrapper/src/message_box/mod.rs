use std::marker::PhantomData;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{GetLastError, E_UNEXPECTED},
        UI::WindowsAndMessaging::{
            MessageBoxW, MB_ICONASTERISK, MB_ICONERROR, MB_ICONEXCLAMATION, MB_ICONHAND,
            MB_ICONINFORMATION, MB_ICONQUESTION, MB_ICONSTOP, MB_ICONWARNING, MESSAGEBOX_RESULT,
            MESSAGEBOX_STYLE,
        },
    },
};

mod abort_retry_ignore;
mod cancel_try_again_continue;
mod okay;
mod okay_cancel;
mod retry_cancel;
mod yes_no;
mod yes_no_cancel;

pub use abort_retry_ignore::*;
pub use cancel_try_again_continue::*;
pub use okay::*;
pub use okay_cancel::*;
pub use retry_cancel::*;
pub use yes_no::*;
pub use yes_no_cancel::*;

pub trait MessageBoxOption: From<MESSAGEBOX_RESULT> {
    fn flags() -> MESSAGEBOX_STYLE;
}

pub enum MessageBoxIcon {
    Exclamation,
    Warning,
    Information,
    Asterisk,
    Question,
    Stop,
    Error,
    Hand,
}

impl MessageBoxIcon {
    fn style(self) -> MESSAGEBOX_STYLE {
        match self {
            MessageBoxIcon::Exclamation => MB_ICONEXCLAMATION,
            MessageBoxIcon::Warning => MB_ICONWARNING,
            MessageBoxIcon::Information => MB_ICONINFORMATION,
            MessageBoxIcon::Asterisk => MB_ICONASTERISK,
            MessageBoxIcon::Question => MB_ICONQUESTION,
            MessageBoxIcon::Stop => MB_ICONSTOP,
            MessageBoxIcon::Error => MB_ICONERROR,
            MessageBoxIcon::Hand => MB_ICONHAND,
        }
    }
}

pub struct MessageBox<T> {
    icon: MessageBoxIcon,
    text: PCWSTR,
    title: Option<PCWSTR>,
    _response: PhantomData<T>,
}

macro_rules! ctors {
    ($($ctor:ident, $name:ident => $icon:ident),*) => {
        $(pub fn $ctor(self) -> Self {
            self.icon(MessageBoxIcon::$icon)
        }
        pub fn $name(text: impl Into<PCWSTR>) -> Self {
            Self::new(text).$ctor()
        }
        )*
    };
}

impl<T: MessageBoxOption> MessageBox<T> {
    pub fn new(text: impl Into<PCWSTR>) -> Self {
        Self {
            icon: MessageBoxIcon::Information,
            text: text.into(),
            title: None,
            _response: Default::default(),
        }
    }

    pub fn icon(mut self, icon: MessageBoxIcon) -> Self {
        self.icon = icon;
        self
    }

    ctors! {
      with_exclamation, exclamation => Exclamation,
      with_warning, warning => Warning,
      with_information, information => Information,
      with_asterisk, asterisk => Asterisk,
      with_question, question => Question,
      with_stop, stop => Stop,
      with_error, error => Error,
      with_hand, hand => Hand
    }

    pub fn with_title(mut self, title: impl Into<PCWSTR>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn show(self) -> windows::core::Result<T> {
        let return_code = unsafe {
            if let Some(title) = self.title {
                MessageBoxW(None, self.text, title, T::flags() | self.icon.style())
            } else {
                MessageBoxW(None, self.text, None, T::flags() | self.icon.style())
            }
        };
        match return_code {
            MESSAGEBOX_RESULT(0) => Err(unsafe {
                match GetLastError() {
                    Err(e) => e,
                    Ok(_) => E_UNEXPECTED.into(),
                }
            }),
            x => Ok(T::from(x)),
        }
    }
}
