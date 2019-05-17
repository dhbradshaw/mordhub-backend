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

use actix_files as fs;
use actix_web::{
    guard,
    http::StatusCode,
    middleware,
    middleware::identity::{CookieIdentityPolicy, IdentityService},
    web, App, Either, HttpResponse, HttpServer,
};
use diesel::{r2d2::ConnectionManager, PgConnection};
use dotenv::dotenv;
use reqwest::r#async::Client;

pub type Or404<T> = Either<T, actix_web::Result<fs::NamedFile>>;

pub fn handle_404() -> actix_web::Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

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
            // Index
            .route("/", web::get().to(routes::index::index))
            // TODO: Service
            .route("/auth/login", web::get().to(routes::auth::login))
            .route("/auth/logout", web::get().to(routes::auth::logout))
            .route("/auth/callback", web::get().to(routes::auth::callback))
            .route("/user/{id}", web::get().to(routes::user::user_profile))
            .default_service(
                web::resource("").route(web::get().to(handle_404)).route(
                    web::route()
                        .guard(guard::Not(guard::Get()))
                        .to(HttpResponse::MethodNotAllowed),
                ),
            )
    })
    .bind("localhost:3000")
    .expect("can't bind to localhost:3000")
    .start();

    system.run().expect("system run error");
}
