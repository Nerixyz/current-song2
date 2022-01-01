pub mod autostart;
pub mod elevate;
pub mod message_box;
mod pwstr;
pub mod path;

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
