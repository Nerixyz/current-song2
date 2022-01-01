use std::env;

/// Sets the working directory to the directory of the executable
pub fn cd_to_exe() {
    // the executable always exists
    let current_exe = env::current_exe().unwrap();
    // the executable always has a parent directory
    env::set_current_dir(current_exe.parent().unwrap()).unwrap();
}
