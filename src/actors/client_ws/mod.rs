use crate::manager;
use futures::StreamExt;
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;
use tracing::{error, event, Level};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(40);

pub async fn handle(
    mut session: actix_ws::Session,
    mut messages: actix_ws::AggregatedMessageStream,
    rx: watch::Receiver<manager::Event>,
) {
    let mut ticker = tokio::time::interval(HEARTBEAT_INTERVAL);
    let mut last_hb = Instant::now();
    let mut stream = WatchStream::new(rx);
    loop {
        let res = tokio::select! {
            _ = ticker.tick() => {
                if Instant::now().duration_since(last_hb) > CLIENT_TIMEOUT {
                    let _ = session.close(Some(actix_ws::CloseReason{ code: actix_ws::CloseCode::Normal, description: None})).await;
                    break;
                } else {
                    session.text(serde_json::json!({ "type": "Ping" }).to_string()).await
                }
            },
            Some(msg) = messages.recv() => {
                match msg {
                    Ok(actix_ws::AggregatedMessage::Text(byte_string)) => {
                        let msg = serde_json::from_slice::<Response>(byte_string.as_ref());
                        if let Ok(Response::Pong) = msg {
                            last_hb = Instant::now();
                        }
                        Ok(())
                    },
                    Ok(actix_ws::AggregatedMessage::Ping(bytes)) => {
                        session.pong(&bytes).await
                    },
                    Ok(_)  => Ok(()),
                    Err(e) => {
                        event!(Level::WARN, error=%e, "WebSocket error");
                        Err(actix_ws::Closed)
                    },
                }
            },
            Some(item) = stream.next() => {
                 match serde_json::to_string(&*item) {
                    Ok(json) => session.text(json).await,
                    Err(e) => {error!(error=%e, "Cannot serialize json"); Ok(())},
                }
            },
        };
        if res.is_err() {
            event!(Level::INFO, "WebSocket closed");
            break;
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "data")]
enum Response {
    Pong,
}
