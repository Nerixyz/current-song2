use crate::{session::SessionCommand, Image};
use std::sync::Weak;
use tap::TapFallible;
use tokio::sync::mpsc;
use tracing::{debug, warn};
use windows::{
    core::{AgileReference, Result},
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionMediaProperties,
    },
    Storage::Streams::{DataReader, IRandomAccessStreamWithContentType},
};

macro_rules! bail_opt {
    ($res:expr) => {
        match ($res)? {
            Some(v) => v,
            None => return Ok(None),
        }
    };
}

pub(crate) trait ResultExt<T> {
    fn opt(self) -> Result<Option<T>>;
}

impl<T> ResultExt<T> for Result<T> {
    fn opt(self) -> Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(e) if e.code().is_ok() => Ok(None),
            Err(e) => Err(e),
        }
    }
}

pub(crate) fn request_media_properties_sync(
    loop_tx: Weak<mpsc::UnboundedSender<SessionCommand>>,
    session: AgileReference<GlobalSystemMediaTransportControlsSession>,
) -> Result<Option<()>> {
    debug!("Getting media properties");
    let session = session.resolve()?;
    let media_properties = session.TryGetMediaPropertiesAsync()?.join()?;
    let get_properties = media_properties.clone();
    let image = try_get_thumbnail(&get_properties)
        .tap_err(|e| warn!(error = ?e, "Couldn't get image"))
        .ok()
        .flatten();

    if let Some(loop_tx) = loop_tx.upgrade() {
        loop_tx
            .send(SessionCommand::MediaPropertiesResult(
                Box::new(bail_opt!(media_properties.try_into().opt())),
                image,
            ))
            .ok();
    }
    Ok(Some(()))
}

fn try_get_thumbnail(
    media_properties: &GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Result<Option<Image>> {
    let thumb = bail_opt!(media_properties.Thumbnail().opt());

    let read = thumb.OpenReadAsync()?;
    let stream = read.join()?;
    let content_type = stream.ContentType()?.to_string();
    let data = read_stream_sync(stream)?;
    Ok(Some(Image { content_type, data }))
}

fn read_stream_sync(stream: IRandomAccessStreamWithContentType) -> Result<Vec<u8>> {
    let stream_len = stream
        .Size()
        .tap_err(|e| warn!(error = %e, "Couldn't get the streams size"))?
        as usize;
    let mut data = vec![0u8; stream_len];
    let reader = DataReader::CreateDataReader(&stream)
        .tap_err(|e| warn!(error = %e, "Couldn't create a data reader"))?;
    reader
        .LoadAsync(stream_len as u32)
        .tap_err(|e| warn!(error = %e, "Couldn't start loading async"))?
        .join()
        .tap_err(|e| warn!(error = %e, "Couldn't load async"))?;
    reader
        .ReadBytes(&mut data)
        .tap_err(|e| warn!(error = %e, "Couldn't read the bytes"))?;

    reader.Close().ok();
    stream.Close().ok();

    Ok(data)
}
