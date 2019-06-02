#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate failure;

mod app;
mod models;
mod routes;
mod schema;

use actix_files as fs;
use actix_web::{
    cookie::SameSite,
    guard, middleware,
    middleware::identity::{CookieIdentityPolicy, IdentityService},
    web, App, HttpResponse, HttpServer, ResponseError,
};
use diesel::{r2d2::ConnectionManager, PgConnection};
use dotenv::dotenv;
use reqwest::r#async::Client;

fn main() {
    std::env::set_var("RUST_LOG", "mordhub=debug,actix_web=info");

    dotenv().ok();
    env_logger::init();

    let system = actix_rt::System::new("MordHub");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("failed to create db pool");

    HttpServer::new(move || {
        let state = app::State {
            pool: pool.clone(),
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
                .same_site(SameSite::Lax) // CSRF mitigation (TODO: add form token mitigation as well)
                .secure(false), // TODO: Use TLS and make this true
            ))
            // Meta
            .route("/", web::get().to(routes::meta::index))
            .route("/about", web::get().to(routes::meta::about))
            // Auth
            .route("/auth/login", web::get().to(routes::auth::login))
            .route("/auth/logout", web::get().to(routes::auth::logout))
            .route(
                "/auth/callback",
                web::get().to_async(routes::auth::callback),
            )
            // User
            .route(
                "/users/{id}",
                web::get().to_async(routes::user::user_profile),
            )
            // Loadouts
            .route("/loadouts", web::get().to_async(routes::loadout::list))
            .route(
                "/loadouts/create",
                web::get().to(routes::loadout::create_get),
            )
            .route(
                "/loadouts/create",
                web::post().to_async(routes::loadout::create_post),
            )
            .route(
                "/loadouts/{id}",
                web::get().to_async(routes::loadout::single),
            )
            // Guides
            .route("/guides", web::get().to(routes::guides::list))
            .service(routes::gen::guides::scope())
            // API
            .route("/api/test", web::get().to(routes::api::test))
            // Static files
            .service(fs::Files::new("/static", "./static/").index_file("404.html"))
            // 404
            .default_service(
                web::resource("")
                    .route(web::get().to(|| app::Error::NotFound.render_response()))
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind("0.0.0.0:3000")
    .expect("can't bind to 0.0.0.0:3000")
    .start();

    println!("Starting server on 0.0.0.0:3000");

    system.run().expect("system run error");
}
