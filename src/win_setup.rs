use std::path::Path;

use crate::{
    config::{self, save_config, Config},
    CONFIG,
};
use win_wrapper::{
    autostart::{add_self_to_autostart, check_autostart, remove_autostart, ERROR_ACCESS_DENIED},
    elevate::elevate_self,
    message_box::{CancelTryAgainContinue, MessageBox, Okay, YesNo},
    single_instance,
};
use windows::core::{w, HSTRING, PCWSTR};

#[cfg(debug_assertions)]
const APPLICATION_NAME: PCWSTR = w!("CurrentSong2Dev");
#[cfg(not(debug_assertions))]
const APPLICATION_NAME: PCWSTR = w!("CurrentSong2");

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
        match remove_autostart(APPLICATION_NAME) {
            Ok(()) => MessageBox::<Okay>::information(w!("Removed from autostart, exiting."))
                .with_title(APPLICATION_NAME)
                .show(),
            Err(e) => {
                // this must be kept alive for the message box
                // TODO: messagebox should make sure this is the case
                let text =
                    HSTRING::from(format!("Failed to remove autostart entry ({e}), exiting."));
                MessageBox::<Okay>::warning(PCWSTR(text.as_ptr()))
                    .with_title(APPLICATION_NAME)
                    .show()
            }
        }
        .ok();
        std::process::exit(0);
    }

    handle_multiple_instances();

    if CONFIG.no_autostart || check_autostart(APPLICATION_NAME) {
        return;
    }
    if MessageBox::question(w!(
        "Add application to autostart?\nYou can remove the entry with --remove-autostart"
    ))
    .with_title(APPLICATION_NAME)
    .show()
    .unwrap_or(YesNo::No)
        == YesNo::No
    {
        let updated_config = Config {
            no_autostart: true,
            ..CONFIG.clone()
        };
        if let Err(e) = save_config(&updated_config, config::current_config_path()) {
            let error = HSTRING::from(format!("Cannot save config, you need to add 'no_autostart = true' to the config.toml.\nError: {e}"));
            MessageBox::<Okay>::error(PCWSTR(error.as_ptr()))
                .with_title(APPLICATION_NAME)
                .show()
                .ok();
        }
        return;
    }

    match add_self_to_autostart(APPLICATION_NAME) {
        Err(e) if e.code() == ERROR_ACCESS_DENIED.to_hresult() => {
            if let Err(e) = elevate_self() {
                let error = HSTRING::from(format!("Cannot elevate process: {e}"));
                MessageBox::<Okay>::error(PCWSTR(error.as_ptr()))
                    .with_title(APPLICATION_NAME)
                    .show()
                    .ok();
            }
        }
        Err(e) => {
            let error = HSTRING::from(format!(
                "Cannot add {} to autostart: {}",
                unsafe { APPLICATION_NAME.display() },
                e
            ));
            MessageBox::<Okay>::error(PCWSTR(error.as_ptr()))
                .with_title(APPLICATION_NAME)
                .show()
                .ok();
        }
        Ok(()) => {
            MessageBox::<Okay>::information(w!("Added to autostart.\nStarting in normal mode."))
                .with_title(APPLICATION_NAME)
                .show()
                .ok();
        }
    };
}

pub fn should_replace_invalid_config(loc: &Path, err: &impl std::fmt::Display) -> bool {
    let error = windows::core::HSTRING::from(format!(
        "Config at {0} was invalid:\n{err}\n\nWhen continuing, {0} will be replaced with the default config.\n",
        loc.display(),
    ));
    match MessageBox::<CancelTryAgainContinue>::error(PCWSTR(error.as_ptr()))
        .with_title(APPLICATION_NAME)
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

    if MessageBox::<YesNo>::information(w!(
        "Another instance is already running. Kill the other instance?"
    ))
    .with_title(APPLICATION_NAME)
    .show()
    .unwrap_or(YesNo::No)
        == YesNo::Yes
    {
        match single_instance::kill_other_instances_of_this_application() {
            Ok(()) => (),
            Err(e) => {
                let error = HSTRING::from(format!("Could not kill the other instance: {e:?}"));
                MessageBox::<Okay>::error(PCWSTR(error.as_ptr()))
                    .with_title(APPLICATION_NAME)
                    .show()
                    .ok();
            }
        }
    }
}

fn elevated_main() -> ! {
    if let Err(e) = add_self_to_autostart(APPLICATION_NAME) {
        let error = HSTRING::from(format!(
            "Cannot add {} to autostart (even in elevated mode): {e}",
            unsafe { APPLICATION_NAME.display() }
        ));
        MessageBox::<Okay>::error(PCWSTR(error.as_ptr()))
            .with_title(APPLICATION_NAME)
            .show()
            .ok();
    } else {
        MessageBox::<Okay>::information(w!(
            "Added to autostart.\nRunning application in user-mode."
        ))
        .with_title(APPLICATION_NAME)
        .show()
        .ok();
    }
    std::process::exit(0)
}
