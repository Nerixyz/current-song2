use crate::{
    config::{save_config, Config},
    CONFIG,
};
use win_wrapper::{
    autostart::{add_self_to_autostart, check_autostart, remove_autostart, ERROR_ACCESS_DENIED},
    elevate::elevate_self,
    message_box::{MessageBox, Okay, YesNo},
    single_instance,
};

#[cfg(debug_assertions)]
const APPLICATION_NAME: &str = "CurrentSong2Dev";
#[cfg(not(debug_assertions))]
const APPLICATION_NAME: &str = "CurrentSong2";

fn has_arg(arg: &str) -> bool {
    std::env::args().any(|a| a == arg)
}

pub fn win_main() {
    // consider using `clap`
    // not _really_ needed since it's very basic
    if has_arg("--elevated") {
        elevated_main();
    }
    if has_arg("--remove-autostart") {
        remove_autostart(APPLICATION_NAME);
        MessageBox::<Okay>::information("Removed from autostart, exiting.")
            .with_title(APPLICATION_NAME)
            .show()
            .ok();
        std::process::exit(0);
    }

    handle_multiple_instances();

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
                MessageBox::<Okay>::error(&format!("Cannot elevate process: {}", e.0))
                    .with_title(APPLICATION_NAME)
                    .show()
                    .ok();
            }
        }
        Err(e) => {
            MessageBox::<Okay>::error(&format!(
                "Cannot add {} to autostart: WindowsErrorCode({})",
                APPLICATION_NAME, e.0
            ))
            .with_title(APPLICATION_NAME)
            .show()
            .ok();
        }
        Ok(_) => {
            MessageBox::<Okay>::information("Added to autostart.\nStarting in normal mode.")
                .with_title(APPLICATION_NAME)
                .show()
                .ok();
        }
    };
}

fn handle_multiple_instances() {
    // consider using something random?
    // not dependant on the version!
    if !single_instance::try_create_new_instance(&format!(
        "current-song2::main-executable::{}",
        CONFIG.server.port
    )) {
        if MessageBox::<YesNo>::information(
            "Another instance is already running. Kill the other instance?",
        )
        .with_title(APPLICATION_NAME)
        .show()
        .unwrap_or(YesNo::No)
            == YesNo::Yes
        {
            match single_instance::kill_other_instances_of_this_application() {
                Ok(_) => (),
                Err(e) => {
                    MessageBox::<Okay>::error(&format!(
                        "Could not kill the other instance: {:?}",
                        e
                    ))
                    .with_title(APPLICATION_NAME)
                    .show()
                    .ok();
                }
            }
        }
    }
}

fn elevated_main() -> ! {
    if let Err(e) = add_self_to_autostart(APPLICATION_NAME) {
        MessageBox::<Okay>::error(&format!(
            "Cannot add {} to autostart (even in elevated mode): WindowsErrorCode({})",
            APPLICATION_NAME, e.0
        ))
        .with_title(APPLICATION_NAME)
        .show()
        .ok();
    } else {
        MessageBox::<Okay>::information("Added to autostart.\nRunning application in user-mode.")
            .with_title(APPLICATION_NAME)
            .show()
            .ok();
    }
    std::process::exit(0)
}
