use std::marker::PhantomData;
use windows::Win32::{
    Foundation::{GetLastError, WIN32_ERROR},
    UI::WindowsAndMessaging::{
        MessageBoxA, MB_ICONASTERISK, MB_ICONERROR, MB_ICONEXCLAMATION, MB_ICONHAND,
        MB_ICONINFORMATION, MB_ICONQUESTION, MB_ICONSTOP, MB_ICONWARNING, MESSAGEBOX_RESULT,
        MESSAGEBOX_STYLE,
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

pub struct MessageBox<'text, 'title, T> {
    icon: MessageBoxIcon,
    text: &'text str,
    title: Option<&'title str>,
    _response: PhantomData<T>,
}

impl<'text, 'title, T: MessageBoxOption> MessageBox<'text, 'title, T> {
    pub fn new(text: &'text str) -> Self {
        Self {
            icon: MessageBoxIcon::Information,
            text,
            title: None,
            _response: Default::default(),
        }
    }

    pub fn icon(mut self, icon: MessageBoxIcon) -> Self {
        self.icon = icon;
        self
    }
    pub fn with_exclamation(self) -> Self {
        self.icon(MessageBoxIcon::Exclamation)
    }
    pub fn with_warning(self) -> Self {
        self.icon(MessageBoxIcon::Warning)
    }
    pub fn with_information(self) -> Self {
        self.icon(MessageBoxIcon::Information)
    }
    pub fn with_asterisk(self) -> Self {
        self.icon(MessageBoxIcon::Asterisk)
    }
    pub fn with_question(self) -> Self {
        self.icon(MessageBoxIcon::Question)
    }
    pub fn with_stop(self) -> Self {
        self.icon(MessageBoxIcon::Stop)
    }
    pub fn with_error(self) -> Self {
        self.icon(MessageBoxIcon::Error)
    }
    pub fn with_hand(self) -> Self {
        self.icon(MessageBoxIcon::Hand)
    }

    pub fn exclamation(text: &'text str) -> Self {
        Self::new(text).with_exclamation()
    }
    pub fn warning(text: &'text str) -> Self {
        Self::new(text).with_warning()
    }
    pub fn information(text: &'text str) -> Self {
        Self::new(text).with_information()
    }
    pub fn asterisk(text: &'text str) -> Self {
        Self::new(text).with_asterisk()
    }
    pub fn question(text: &'text str) -> Self {
        Self::new(text).with_question()
    }
    pub fn stop(text: &'text str) -> Self {
        Self::new(text).with_stop()
    }
    pub fn error(text: &'text str) -> Self {
        Self::new(text).with_error()
    }
    pub fn hand(text: &'text str) -> Self {
        Self::new(text).with_hand()
    }

    pub fn with_title(mut self, title: &'title str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn show(self) -> Result<T, WIN32_ERROR> {
        let return_code = unsafe {
            if let Some(title) = self.title {
                MessageBoxA(None, self.text, title, T::flags() | self.icon.style())
            } else {
                MessageBoxA(None, self.text, None, T::flags() | self.icon.style())
            }
        };
        match return_code {
            MESSAGEBOX_RESULT(0) => Err(unsafe { GetLastError() }),
            x => Ok(T::from(x)),
        }
    }
}
