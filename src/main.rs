#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

mod app;
mod models;
mod schema;

use crate::models::DbExecutor;
use actix::prelude::*;
use actix_web::{App, HttpServer, middleware};
use diesel::{r2d2::ConnectionManager, PgConnection};
use dotenv::dotenv;

fn main() {
    dotenv().ok();

    std::env::set_var("RUST_LOG", "mordhub=debug,actix_web=info");
    env_logger::init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let system = actix::System::new("Mordhub");

    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("failed to create pool");

    let address: Addr<DbExecutor> = SyncArbiter::start(4, move || DbExecutor(pool.clone()));

    HttpServer::new(move || {
        App::new()
            .data(app::AppState { db: address.clone() })
            .wrap(middleware::Logger::new("\"%r\" %s %b %Dms"))
    }).bind("localhost:3000").expect("can't bind to localhost:3000").start();

    system.run().expect("system run error");
}
