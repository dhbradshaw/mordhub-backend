// Forked from https://github.com/OneSignal/L3-37/tree/master/l337-postgres
// See licenses in vendor/l337

use futures::{sync::oneshot, Async, Future, Stream};
use tokio_postgres::{
    error::Error,
    tls::{MakeTlsConnect, TlsConnect},
    Client,
    Socket,
};

use super::queries::Queries;

use std::fmt;

pub struct Connection {
    pub client: Client,
    pub queries: Queries,
    broken: bool,
    receiver: oneshot::Receiver<bool>,
}

pub struct PostgresConnectionManager<T>
where
    T: 'static + MakeTlsConnect<Socket> + Clone + Send + Sync,
{
    config: tokio_postgres::Config,
    make_tls_connect: T,
}

impl<T> PostgresConnectionManager<T>
where
    T: 'static + MakeTlsConnect<Socket> + Clone + Send + Sync,
{
    pub fn new(config: tokio_postgres::Config, make_tls_connect: T) -> Self {
        Self {
            config,
            make_tls_connect,
        }
    }
}

impl<T> l337::ManageConnection for PostgresConnectionManager<T>
where
    T: 'static + MakeTlsConnect<Socket> + Clone + Send + Sync,
    T::Stream: Send + Sync,
    T::TlsConnect: Send,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send + Sync,
{
    type Connection = Connection;
    type Error = Error;

    fn connect(
        &self,
    ) -> Box<Future<Item = Self::Connection, Error = l337::Error<Self::Error>> + 'static + Send>
    {
        Box::new(
            self.config
                .connect(self.make_tls_connect.clone())
                .and_then(|(client, connection)| {
                    let (sender, receiver) = oneshot::channel();
                    actix_rt::spawn(connection.map_err(|_| {
                        sender
                            .send(true)
                            .unwrap_or_else(|e| panic!("failed to send shutdown notice: {}", e));
                    }));
                    Queries::prepare(client).map(move |(client, q)| (client, receiver, q))
                })
                .map(|(client, receiver, queries)| Connection {
                    broken: false,
                    client,
                    receiver,
                    queries,
                })
                .map_err(|e| l337::Error::External(e)),
        )
    }

    fn is_valid(
        &self,
        mut conn: Self::Connection,
    ) -> Box<Future<Item = (), Error = l337::Error<Self::Error>>> {
        Box::new(
            conn.client
                .simple_query("")
                .into_future()
                .map(|_| ())
                .map_err(|(e, _)| l337::Error::External(e)),
        )
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        if conn.broken {
            return true;
        }

        match conn.receiver.poll() {
            Ok(Async::Ready(_)) => {
                conn.broken = true;
                true
            }
            Ok(Async::NotReady) => false,
            Err(err) => panic!("polling oneshot failed: {}", err),
        }
    }

    fn timed_out(&self) -> l337::Error<Self::Error> {
        unimplemented!()
        // Error::io(io::ErrorKind::TimedOut.into())
    }
}

impl<T> fmt::Debug for PostgresConnectionManager<T>
where
    T: 'static + MakeTlsConnect<Socket> + Clone + Send + Sync,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PostgresConnectionManager")
            .field("config", &self.config)
            .finish()
    }
}
