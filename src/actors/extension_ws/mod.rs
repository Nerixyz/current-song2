use crate::{
    actors::manager::{self, Manager},
    model::PlayInfo,
};
use actix::Addr;
use serde::Deserialize;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{event, warn, Level};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(40);

pub async fn handle(
    mut session: actix_ws::Session,
    mut messages: actix_ws::AggregatedMessageStream,
    manager: Arc<Addr<Manager>>,
) {
    let mut ticker = tokio::time::interval(HEARTBEAT_INTERVAL);
    let mut last_hb = Instant::now();
    let Ok(id) = manager
        .send(manager::CreateModule { priority: 1 })
        .await
        .inspect_err(|e| warn!(error=%e, "Failed to request module"))
    else {
        return;
    };
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
            Some(Ok(msg)) = messages.recv() => {
                match msg {
                    actix_ws::AggregatedMessage::Text(byte_string) => {
                        match serde_json::from_slice::<Response>(byte_string.as_ref()) {
                            Ok(Response::Active(info)) => {
                                manager.do_send(manager::UpdateModule::playing(id, info));
                            },
                            Ok(Response::Inactive)=> {
                                manager.do_send(manager::UpdateModule::paused(id));
                            },
                            Ok(Response::Pong) => {
                                last_hb = Instant::now();
                            }
                            Err(e) => {
                                warn!(error=%e, id = %id, "Failed to deserialize");
                            }
                        };
                        Ok(())
                    },
                    actix_ws::AggregatedMessage::Ping(bytes) => {
                        session.pong(&bytes).await
                    },
                    _  => Ok(()),
                }
            },
            else => {
                event!(Level::DEBUG, id = %id, "WebSocket closed");
                break;
            }
        };
        if res.is_err() {
            event!(Level::INFO, id = %id, "WebSocket closed");
            break;
        }
    }

    manager.do_send(manager::RemoveModule { id });
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "data")]
enum Response {
    Pong,
    Active(PlayInfo),
    Inactive,
}
