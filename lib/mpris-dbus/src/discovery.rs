use futures::StreamExt;
use zbus::Connection;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Failed to get DBus connection to the user message bus ({0})")]
    GetConnection(zbus::Error),
    #[error("Failed to setup the proxy ({0})")]
    SetupProxy(zbus::Error),
    #[error("Failed to listen to the name acquired stream ({0})")]
    ListenNameAcquired(zbus::Error),
    #[error("Failed to list names ({0})")]
    ListNames(zbus::fdo::Error),
}

pub struct Listener {
    conn: Connection,
}

impl Listener {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self {
            conn: Connection::session().await.map_err(Error::GetConnection)?,
        })
    }

    pub async fn listen(
        &self,
    ) -> Result<impl futures::Stream<Item = zbus_names::BusName<'static>>, Error> {
        let proxy = zbus::fdo::DBusProxy::new(&self.conn)
            .await
            .map_err(Error::SetupProxy)?;

        let current = proxy.list_names().await.map_err(Error::ListNames)?;

        Ok(
            futures::stream::iter(current.into_iter().map(|it| it.into_inner())).chain(
                proxy
                    .receive_name_owner_changed()
                    .await
                    .map_err(Error::ListenNameAcquired)?
                    .filter_map(|it| {
                        std::future::ready(it.args().ok().and_then(|n| {
                            if n.new_owner.is_some() {
                                Some(n.name.into_owned())
                            } else {
                                None
                            }
                        }))
                    }),
            ),
        )
    }
}
