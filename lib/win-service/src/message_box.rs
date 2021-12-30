use windows::Win32::UI::Shell::ShellMessageBoxW;

pub fn message_box(content: &str) {
    unsafe {
        ShellMessageBoxW(0, None, content, None, 0);
    }
}
