use actix_web_static_files::deps::static_files::{resource_dir, NpmBuild};
use std::{env, io, path::PathBuf};

fn include_static_files() -> io::Result<()> {
    // assume it's already built
    if PathBuf::from("js/client/dist").exists() {
        resource_dir("js/client/dist").build()
    } else {
        NpmBuild::new("js")
            .install()?
            .run("build-client")?
            .target("js/client/dist")
            .change_detection()
            .to_resource_dir()
            .build()
    }
}

fn main() -> io::Result<()> {
    if env::var("CARGO_FEATURE_SINGLE_EXECUTABLE").is_ok() {
        include_static_files()?;
    }
    Ok(())
}
