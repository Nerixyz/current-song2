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

pub async fn request_media_properties(
    loop_tx: Weak<mpsc::UnboundedSender<SessionCommand>>,
    session: AgileReference<GlobalSystemMediaTransportControlsSession>,
) -> Result<()> {
    debug!("Getting new image");
    let session = session.resolve()?;
    let media_properties = session.TryGetMediaPropertiesAsync()?.await?;
    let get_properties = media_properties.clone();
    let image = tokio::task::spawn_blocking(move || try_get_thumbnail_sync(&get_properties))
        .await
        .tap_err(|e| error!(error = %e,"Couldn't read stream"))
        .map_err(|_| windows::core::Error::from(E_ABORT))?
        .tap_err(|e| warn!(error = %e, "Couldn't get image"))
        .ok();

    if let Some(loop_tx) = loop_tx.upgrade() {
        loop_tx
            .send(SessionCommand::MediaPropertiesResult(
                media_properties.try_into()?,
                image,
            ))
            .ok();
    }
    Ok(())
}

fn try_get_thumbnail_sync(
    media_properties: &GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Result<Image> {
    let read = media_properties.Thumbnail()?.OpenReadAsync()?;
    let stream = read.get()?;
    let content_type = stream.ContentType()?.to_string();
    let data = read_stream_sync(stream)?;
    Ok(Image { content_type, data })
}

fn read_stream_sync(stream: IRandomAccessStreamWithContentType) -> Result<Vec<u8>> {
    let stream_len = stream.Size()? as usize;
    let mut data = vec![0u8; stream_len];
    let reader = DataReader::CreateDataReader(&stream)?;
    reader.LoadAsync(stream_len as u32)?.get()?;
    reader.ReadBytes(&mut data)?;

    reader.Close().ok();
    stream.Close().ok();

    Ok(data)
}
