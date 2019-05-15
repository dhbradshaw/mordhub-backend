use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use reqwest::r#async::Client;
use tera::Tera;

pub struct AppState {
    pub pool: Pool<ConnectionManager<PgConnection>>,
    pub tera: Tera,
    pub reqwest: Client,
}
