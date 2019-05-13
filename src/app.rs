use crate::models::DbExecutor;
use actix::prelude::*;
use reqwest::r#async::Client;
use tera::Tera;

pub struct AppState {
    pub db: Addr<DbExecutor>,
    pub tera: Tera,
    pub reqwest: Client,
}
