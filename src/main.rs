#[macro_use]
extern crate diesel;
#[macro_use]
extern crate tera;
#[macro_use]
extern crate log;

mod app;
mod models;
mod routes;
mod schema;

use actix::prelude::*;
use app::AppState;
use diesel::{r2d2::ConnectionManager, PgConnection};
use dotenv::dotenv;
use reqwest;
use actix_web::{
    middleware,
    middleware::identity::{CookieIdentityPolicy, IdentityService},
    web, App, HttpServer,
};

use models::DbExecutor;

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
        let tera = compile_templates!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"));

        let state = AppState {
            db: address.clone(),
            tera,
        };

        App::new()
            .data(state)
            .wrap(middleware::Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(
                    std::env::var("COOKIE_SECRET")
                        .expect("COOKIE_SECRET must be set")
                        .as_bytes(),
                )
                .name("auth-cookie")
                .secure(false), // TODO: Use TLS and make this true
            ))
            .route("/", web::get().to(routes::index::index))
            .route("/login", web::get().to(routes::auth::login))
    })
    .bind("localhost:3000")
    .expect("can't bind to localhost:3000")
    .start();

    system.run().expect("system run error");
}
