//! This library is a wrapper around the [`Windows.Media.Control`](https://learn.microsoft.com/uwp/api/windows.media.control) namespace (aka `GlobalSystemMediaTransportControls` - GSMTC).
//! It uses [`tokio`](https://docs.rs/tokio) to manage internal workers that deliver updates.
//!
//! ### Example
//!
//! ```no_run
//! # use windows::core::Result;
//! # #[tokio::main]
//! # async fn main() -> Result<()> {
//! use gsmtc::{ManagerEvent::*, SessionUpdateEvent::*};
//!
//! let mut rx = gsmtc::SessionManager::create().await?;
//!
//! while let Some(evt) = rx.recv().await {
//!     match evt {
//!         SessionCreated {
//!             session_id,
//!             mut rx,
//!             source,
//!         } => {
//!             println!("Created session: {{id={session_id}, source={source}}}");
//!             tokio::spawn(async move {
//!                 while let Some(evt) = rx.recv().await {
//!                     match evt {
//!                         Model(model) => {
//!                             println!("[{session_id}/{source}] Model updated: {model:#?}")
//!                         }
//!                         Media(model, image) => println!(
//!                             "[{session_id}/{source}] Media updated: {model:#?} - {image:?}"
//!                         ),
//!                     }
//!                 }
//!                 println!("[{session_id}/{source}] exited event-loop");
//!             });
//!         }
//!         SessionRemoved { session_id } => println!("Session {{id={session_id}}} was removed"),
//!         CurrentSessionChanged {
//!             session_id: Some(id),
//!         } => println!("Current session: {id}"),
//!         CurrentSessionChanged { session_id: None } => println!("No more current session"),
//!     }
//! }
//! # Ok(())
//! # }
//! ```
mod conversion;
mod manager;
mod model;
mod session;
mod util;

pub use manager::{ManagerEvent, SessionManager};
pub use model::*;
pub use session::SessionUpdateEvent;

pub(crate) type EventRegistrationToken = i64;
