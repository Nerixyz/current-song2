#[cfg(feature = "single-executable")]
mod static_web_files {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

#[cfg(feature = "single-executable")]
pub fn service() -> actix_web_static_files::ResourceFiles {
    let generated = static_web_files::generate();
    actix_web_static_files::ResourceFiles::new("", generated).resolve_not_found_to_root()
}

#[cfg(not(feature = "single-executable"))]
pub fn service() -> actix_files::Files {
    actix_files::Files::new("/", "js/packages/client/dist").index_file("index.html")
}