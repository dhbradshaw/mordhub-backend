#[macro_use]
extern crate diesel;
#[macro_use]
extern crate tera;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate futures;

mod app;
mod models;
mod routes;
mod schema;

use actix::prelude::*;
use actix_web::{
    middleware,
    middleware::identity::{CookieIdentityPolicy, IdentityService},
    web, App, HttpServer,
};
use app::AppState;
use diesel::{r2d2::ConnectionManager, PgConnection};
use dotenv::dotenv;
use reqwest::r#async::Client;

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

    let _site_url = std::env::var("SITE_URL").expect("SITE_URL must be set");

    HttpServer::new(move || {
        let tera = compile_templates!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"));

        let state = AppState {
            db: address.clone(),
            tera,
            reqwest: Client::new(), // TODO: Initialise TLS
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
            .route("/auth/login", web::get().to(routes::auth::login))
            .route("/auth/logout", web::get().to(routes::auth::logout))
            .route("/auth/callback", web::get().to(routes::auth::callback))
    })
    .bind("localhost:3000")
    .expect("can't bind to localhost:3000")
    .start();

    system.run().expect("system run error");
}
