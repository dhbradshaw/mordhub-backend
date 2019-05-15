use diesel::{
    r2d2::{PooledConnection, ConnectionManager, Pool},
    PgConnection,
};
use reqwest::r#async::Client;
use tera::Tera;

pub struct AppState {
    pub pool: Pool<ConnectionManager<PgConnection>>,
    pub tera: Tera,
    pub reqwest: Client,
}

impl AppState {
    pub fn get_conn(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.get().unwrap()
    }
}
