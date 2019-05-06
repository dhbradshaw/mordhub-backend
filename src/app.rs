use actix::prelude::*;

use crate::models::DbExecutor;

pub struct AppState {
    pub db: Addr<DbExecutor>,
}
