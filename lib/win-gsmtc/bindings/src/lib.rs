windows::include_bindings!();

// TODO: check for a better solution
unsafe impl Send for Windows::Storage::Streams::IRandomAccessStreamWithContentType {}
