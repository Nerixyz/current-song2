use std::env;

/// Sets the working directory to the directory of the executable.
/// Skips if --current-dir is specified
pub fn cd_to_exe() {
    if env::args().any(|a| a == "--current-dir") {
        return;
    }

    // the executable always exists
    let current_exe = env::current_exe().unwrap();
    // the executable always has a parent directory
    env::set_current_dir(current_exe.parent().unwrap()).unwrap();
}
