use actix_web_static_files::deps::static_files::resource_dir;
use std::{env, io, path::PathBuf};

fn include_static_files() -> io::Result<()> {
    // assume it's already built
    if PathBuf::from("js/client/dist").exists() {
        resource_dir("js/client/dist").build()
    } else {
        panic!("single-executable feature enabled, but js/client/dist doesn't exist. Did you forget to run `pnpm build` in `js/client`?")
    }
}

fn main() -> io::Result<()> {
    if env::var("CARGO_FEATURE_SINGLE_EXECUTABLE").is_ok() {
        include_static_files()?;
    }
    Ok(())
}
