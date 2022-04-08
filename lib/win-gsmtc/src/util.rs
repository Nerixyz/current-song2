use crate::{session::SessionCommand, Image};
use std::sync::Weak;
use tap::TapFallible;
use tokio::sync::mpsc;
use tracing::{debug, error, warn};
use windows::{
    core::{AgileReference, Result},
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionMediaProperties,
    },
    Storage::Streams::{DataReader, IRandomAccessStreamWithContentType},
    Win32::Foundation::E_ABORT,
};

macro_rules! opt_result {
    ($res:expr) => {
        match optional_result($res)? {
            Some(v) => v,
            None => return Ok(None),
        }
    };
}

pub async fn request_media_properties(
    loop_tx: Weak<mpsc::UnboundedSender<SessionCommand>>,
    session: AgileReference<GlobalSystemMediaTransportControlsSession>,
) -> Result<Option<()>> {
    debug!("Getting media properties");
    let session = session.resolve()?;
    let media_properties = session.TryGetMediaPropertiesAsync()?.await?;
    let get_properties = media_properties.clone();
    let image = tokio::task::spawn_blocking(move || try_get_thumbnail_sync(&get_properties))
        .await
        .tap_err(|e| error!(error = %e,"Couldn't read stream"))
        .map_err(|_| windows::core::Error::from(E_ABORT))?
        .tap_err(|e| warn!(error = ?e, "Couldn't get image"))
        .ok()
        .flatten();

    if let Some(loop_tx) = loop_tx.upgrade() {
        loop_tx
            .send(SessionCommand::MediaPropertiesResult(
                opt_result!(media_properties.try_into()),
                image,
            ))
            .ok();
    }
    Ok(Some(()))
}

fn try_get_thumbnail_sync(
    media_properties: &GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Result<Option<Image>> {
    let thumb = opt_result!(media_properties.Thumbnail());

    let read = thumb.OpenReadAsync()?;
    let stream = read.get()?;
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
        .get()
        .tap_err(|e| warn!(error = %e, "Couldn't load async"))?;
    reader
        .ReadBytes(&mut data)
        .tap_err(|e| warn!(error = %e, "Couldn't read the bytes"))?;

    reader.Close().ok();
    stream.Close().ok();

    Ok(data)
}

pub fn optional_result<T>(result: Result<T>) -> Result<Option<T>> {
    match result {
        Ok(v) => Ok(Some(v)),
        Err(e) if e.code().is_ok() => Ok(None),
        Err(e) => Err(e),
    }
}
