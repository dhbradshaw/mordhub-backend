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
#[macro_use]
extern crate failure;

mod models;
mod routes;
mod schema;
mod state;

use actix_web::{
    middleware,
    middleware::identity::{CookieIdentityPolicy, IdentityService},
    web, App, HttpServer,
};
use diesel::{r2d2::ConnectionManager, PgConnection};
use dotenv::dotenv;
use reqwest::r#async::Client;

fn main() {
    std::env::set_var("RUST_LOG", "mordhub=debug,actix_web=info");

    dotenv().ok();
    env_logger::init();

    let system = actix_rt::System::new("Mordhub");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("failed to create pool");

    HttpServer::new(move || {
        let tera = compile_templates!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"));

        let state = state::AppState {
            pool: pool.clone(),
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
