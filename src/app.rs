use crate::models::DbExecutor;
use actix::prelude::*;
use tera::Tera;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub db: Addr<DbExecutor>,
    pub tera: Tera,
}
