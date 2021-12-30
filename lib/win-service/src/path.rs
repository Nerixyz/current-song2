use std::env;

pub fn cd_to_exe() {
    let current_exe = env::current_exe().unwrap();
    env::set_current_dir(current_exe.parent().unwrap()).unwrap();
}
