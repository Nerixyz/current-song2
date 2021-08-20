fn main() {
    windows::build! {
        Windows::Media::Control::{
            GlobalSystemMediaTransportControlsSession,
            GlobalSystemMediaTransportControlsSessionManager,
            GlobalSystemMediaTransportControlsSessionMediaProperties,
            GlobalSystemMediaTransportControlsSessionPlaybackControls,
            GlobalSystemMediaTransportControlsSessionPlaybackInfo,
            GlobalSystemMediaTransportControlsSessionPlaybackStatus,
            GlobalSystemMediaTransportControlsSessionTimelineProperties,
        },
        Windows::Foundation::Collections::IVectorView,
        Windows::Foundation::{TypedEventHandler, IAsyncOperation, IAsyncOperationWithProgress, TimeSpan, DateTime, IReference},
        Windows::Storage::Streams::{IRandomAccessStreamWithContentType, IRandomAccessStreamReference, DataReader, Buffer},
    }
}
