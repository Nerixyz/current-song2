#[macro_export]
macro_rules! cfg_windows {
    ($($item:item)*) => {
        $( #[cfg(windows)] $item )*
    }
}
