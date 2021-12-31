use crate::{
    config::{save_config, Config},
    CONFIG,
};
use win_wrapper::{
    autostart::{add_self_to_autostart, check_autostart, remove_autostart, ERROR_ACCESS_DENIED},
    elevate::elevate_self,
    message_box::{MessageBox, Okay, YesNo},
};

#[cfg(debug_assertions)]
const APPLICATION_NAME: &str = "CurrentSong2Dev";
#[cfg(not(debug_assertions))]
const APPLICATION_NAME: &str = "CurrentSong2";

fn has_arg(arg: &str) -> bool {
    std::env::args().any(|a| a == arg)
}

pub fn win_main() {
    if has_arg("--elevated") {
        elevated_main();
    }
    if has_arg("--remove-autostart") {
        remove_autostart(APPLICATION_NAME);
        return;
    }

    if CONFIG.no_autostart || check_autostart(APPLICATION_NAME) {
        return;
    }
    if MessageBox::question(
        "Add application to autostart?\nYou can remove the entry with --remove-autostart",
    )
    .with_title(APPLICATION_NAME)
    .show()
    .unwrap_or(YesNo::No)
        == YesNo::No
    {
        let updated_config = Config {
            no_autostart: true,
            ..CONFIG.clone()
        };
        if let Err(e) = save_config(&updated_config) {
            MessageBox::<Okay>::error(&format!("Cannot save config, you need to add 'no_autostart = true' to the config.toml.\nError: {}", e)).with_title(APPLICATION_NAME).show().ok();
        }
        return;
    }

    match add_self_to_autostart(APPLICATION_NAME) {
        Err(ERROR_ACCESS_DENIED) => {
            if let Err(e) = elevate_self() {
                MessageBox::<Okay>::error(&format!("Cannot elevate process: {}", e))
                    .with_title(APPLICATION_NAME)
                    .show()
                    .ok();
            }
        }
        Err(e) => {
            MessageBox::<Okay>::error(&format!(
                "Cannot add {} to autostart: WindowsErrorCode({})",
                APPLICATION_NAME, e
            ))
            .with_title(APPLICATION_NAME)
            .show()
            .ok();
        }
        _ => (),
    };
}

fn elevated_main() -> ! {
    if let Err(e) = add_self_to_autostart(APPLICATION_NAME) {
        MessageBox::<Okay>::error(&format!(
            "Cannot add {} to autostart (even in elevated mode): WindowsErrorCode({})",
            APPLICATION_NAME, e
        ))
        .with_title(APPLICATION_NAME)
        .show()
        .ok();
    }
    std::process::exit(0)
}
