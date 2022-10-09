use crate::{
    manager,
    model::{PlayInfo, TimelineInfo},
    utilities::format_string::{FormatDescription, InterpolationProvider},
    ModuleState, CONFIG,
};
use std::{borrow::Cow, fmt, fmt::Write, path::Path, time::Duration};
use tap::TapFallible;
use tokio::sync::watch;
use tracing::{debug, info, warn};

#[derive(Debug)]
enum Interpolation {
    Title,
    Artist,
    AlbumName,
    AlbumTracks,
    TrackNumber,
    Source,
    Duration,
}

impl InterpolationProvider for Interpolation {
    type Source = PlayInfo;
    type ParseError = anyhow::Error;
    type FormatError = fmt::Error;

    fn parse_provider(name: &str) -> Result<Self, Self::ParseError> {
        match name {
            "title" => Ok(Self::Title),
            "artist" => Ok(Self::Artist),
            "album-name?" => Ok(Self::AlbumName),
            "album-tracks?" => Ok(Self::AlbumTracks),
            "track-number?" => Ok(Self::TrackNumber),
            "source" => Ok(Self::Source),
            "duration?" => Ok(Self::Duration),
            x => Err(anyhow::anyhow!("'{}' is not a valid interpolation", x)),
        }
    }

    fn format<W: Write>(&self, source: &PlayInfo, f: &mut W) -> Result<(), Self::FormatError> {
        match self {
            Interpolation::Title => f.write_str(&source.title),
            Interpolation::Artist => f.write_str(&source.artist),
            Interpolation::AlbumName => match source.album {
                Some(ref album) => f.write_str(&album.title),
                None => Ok(()),
            },
            Interpolation::AlbumTracks => match source.album {
                Some(ref album) if album.track_count > 0 => write!(f, "{}", album.track_count),
                _ => Ok(()),
            },
            Interpolation::TrackNumber => match source.track_number {
                Some(n) if n > 0 => write!(f, "{n}"),
                _ => Ok(()),
            },
            Interpolation::Source => f.write_str(&source.source),
            Interpolation::Duration => match source.timeline {
                Some(TimelineInfo { duration_ms, .. }) => {
                    let duration = Duration::from_millis(duration_ms);
                    write!(
                        f,
                        "{}m{}s",
                        duration.as_secs() / 60,
                        duration.as_secs() % 60
                    )
                }
                None => Ok(()),
            },
        }
    }
}

pub async fn output_to_file<P>(file_path: P, mut rx: watch::Receiver<manager::Event>)
where
    P: AsRef<Path>,
{
    let path = file_path.as_ref();
    debug!(path = ?path, "Enabled output to file");

    let format_descr =
        FormatDescription::<Interpolation>::try_from(Cow::from(&CONFIG.modules.file.format))
            .tap_err(|e| warn!(erorr = %e, "Invalid format"))
            .unwrap_or_else(|_| FormatDescription::raw("invalid format"));

    while rx.changed().await.is_ok() {
        let formatted = format_event(&rx.borrow(), &format_descr);
        if let Err(e) = tokio::fs::write(path, formatted.trim_end().as_bytes()).await {
            warn!(error = %e, "Couldn't write to file");
        }
    }
    info!("Channel closed - Stopped file output");
}

fn format_event(
    state: &ModuleState,
    format_descr: &FormatDescription<Interpolation>,
) -> Cow<'static, str> {
    match state {
        ModuleState::Playing(info) => format_descr
            .format_to_string(info)
            .map_or_else(|_| "cannot format".into(), Cow::from),
        ModuleState::Paused => "".into(),
    }
}
