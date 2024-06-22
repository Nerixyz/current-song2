use std::path::Path;

use crate::{
    config::{self, save_config, Config},
    CONFIG,
};
use win_msgbox::{CancelTryAgainContinue, MessageBox, Okay, YesNo};
use win_wrapper::{
    autostart::{add_self_to_autostart, check_autostart, remove_autostart, ERROR_ACCESS_DENIED},
    elevate::elevate_self,
    single_instance,
};
use windows::core::{w, HSTRING, PCWSTR};

#[cfg(debug_assertions)]
const APPLICATION_NAME: &str = "CurrentSong2Dev";
#[cfg(debug_assertions)]
const APPLICATION_NAME_W: PCWSTR = w!("CurrentSong2Dev");
#[cfg(not(debug_assertions))]
const APPLICATION_NAME: &str = "CurrentSong2";
#[cfg(not(debug_assertions))]
const APPLICATION_NAME_W: PCWSTR = w!("CurrentSong2");

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
        match remove_autostart(APPLICATION_NAME_W) {
            Ok(()) => confirm_information("Removed from autostart, exiting."),
            Err(e) => confirm_warning(&format!("Failed to remove autostart entry ({e}), exiting.")),
        }

        std::process::exit(0);
    }

    handle_multiple_instances();

    if CONFIG.no_autostart || check_autostart(APPLICATION_NAME_W) {
        return;
    }

    let should_add = MessageBox::question(
        "Add application to autostart?\nYou can remove the entry with --remove-autostart",
    )
    .title(APPLICATION_NAME)
    .show()
    .unwrap_or(YesNo::No)
        == YesNo::No;

    if should_add {
        let updated_config = Config {
            no_autostart: true,
            ..CONFIG.clone()
        };
        if let Err(e) = save_config(&updated_config, config::current_config_path()) {
            let error = format!("Cannot save config, you need to add 'no_autostart = true' to the config.toml.\nError: {e}");
            confirm_error(&error);
        }
        return;
    }

    match add_self_to_autostart(APPLICATION_NAME_W) {
        Err(e) if e.code() == ERROR_ACCESS_DENIED.to_hresult() => {
            if let Err(e) = elevate_self() {
                confirm_error(&format!("Cannot elevate process: {e}"));
            }
        }
        Err(e) => {
            confirm_error(&format!("Cannot add {APPLICATION_NAME} to autostart: {e}"));
        }
        Ok(()) => {
            confirm_information("Added to autostart.\nStarting in normal mode.");
        }
    };
}

pub fn should_replace_invalid_config(loc: &Path, err: &impl std::fmt::Display) -> bool {
    let error = format!(
        "Config at {0} was invalid:\n{err}\n\nWhen continuing, {0} will be replaced with the default config.\n",
        loc.display(),
    );
    match MessageBox::error(&error)
        .title(APPLICATION_NAME)
        .show()
        .ok()
    {
        Some(CancelTryAgainContinue::Cancel) => std::process::exit(1),
        Some(CancelTryAgainContinue::TryAgain) => false,
        None | Some(CancelTryAgainContinue::Continue) => true,
    }
}

fn fmt_instance_id() -> String {
    let mut base = "current-song2::main-executable::".to_owned();
    let _ = match &CONFIG.server.bind {
        config::BindConfig::Single { port } => std::fmt::write(&mut base, format_args!("{port}")),
        config::BindConfig::Multiple { bind } => {
            std::fmt::write(&mut base, format_args!("{bind:?}"))
        }
    };
    base
}

fn handle_multiple_instances() {
    // consider using something random?
    // not dependant on the version!
    let instance_id = HSTRING::from(fmt_instance_id());
    if single_instance::try_create_new_instance(&instance_id) {
        return;
    }

    if MessageBox::information("Another instance is already running. Kill the other instance?")
        .title(APPLICATION_NAME)
        .show()
        .unwrap_or(YesNo::No)
        == YesNo::Yes
    {
        match single_instance::kill_other_instances_of_this_application() {
            Ok(()) => (),
            Err(e) => {
                confirm_error(&format!("Could not kill the other instance: {e:?}"));
            }
        }
    }
}

fn elevated_main() -> ! {
    if let Err(e) = add_self_to_autostart(APPLICATION_NAME_W) {
        let error =
            format!("Cannot add {APPLICATION_NAME} to autostart (even in elevated mode): {e}",);
        confirm_error(&error);
    } else {
        confirm_information("Added to autostart.\nRunning application in user-mode.");
    }
    std::process::exit(0)
}

fn confirm_information(msg: &str) {
    MessageBox::<Okay>::information(msg)
        .title(APPLICATION_NAME)
        .show()
        .ok();
}

fn confirm_error(msg: &str) {
    MessageBox::<Okay>::error(msg)
        .title(APPLICATION_NAME)
        .show()
        .ok();
}

fn confirm_warning(msg: &str) {
    MessageBox::<Okay>::warning(msg)
        .title(APPLICATION_NAME)
        .show()
        .ok();
}
