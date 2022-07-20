use crate::{manager, model::PlayInfo, ModuleState};
use std::{borrow::Cow, path::Path};
use tokio::sync::watch;
use tracing::{debug, info, warn};

pub async fn output_to_file<P>(file_path: P, mut rx: watch::Receiver<manager::Event>)
where
    P: AsRef<Path>,
{
    let path = file_path.as_ref();
    debug!(path = ?path, "Enabled output to file");
    while rx.changed().await.is_ok() {
        let formatted = format_event(&*rx.borrow());
        if let Err(e) = tokio::fs::write(path, formatted.as_bytes()).await {
            warn!(error = %e, "Couldn't write to file");
        }
    }
    info!("Channel closed - Stopped file output");
}

fn format_event(state: &ModuleState) -> Cow<'static, str> {
    match state {
        ModuleState::Playing(PlayInfo { title, artist, .. }) => {
            format!("{} - {}", artist, title).into()
        }
        ModuleState::Paused => "".into(),
    }
}
