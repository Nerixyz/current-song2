pub mod autostart;
pub mod elevate;
pub mod message_box;
pub mod path;
mod pwstr;

#[cfg(test)]
mod tests {
    use crate::message_box::{MessageBox, YesNoCancel};

    #[test]
    fn it_works() {
        println!(
            "{:?}",
            MessageBox::<YesNoCancel>::error("forsen")
                .with_title("aliens")
                .show()
        );
    }
}
