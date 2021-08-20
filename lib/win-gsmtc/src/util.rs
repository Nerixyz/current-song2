use bindings::Windows::Storage::Streams::{
    Buffer, DataReader, IRandomAccessStreamWithContentType, InputStreamOptions,
};
use windows::Result;

pub async fn read_stream_entirely(stream: IRandomAccessStreamWithContentType) -> Result<Vec<u8>> {
    let stream_len = stream.Size()? as usize;
    let mut data = vec![0u8; stream_len];
    let buf = Buffer::Create(stream_len as u32)?;

    let read_fut = stream.ReadAsync(&buf, stream_len as u32, InputStreamOptions::None)?;
    let buf = read_fut.await?;

    let reader = DataReader::FromBuffer(buf)?;
    reader.ReadBytes(&mut data)?;

    stream.Close().ok();

    Ok(data)
}
