use crate::models::User;
use actix_web::HttpResponse;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};
use reqwest::r#async::Client;
use tera::{Context, Tera};

pub struct AppState {
    pub pool: Pool<ConnectionManager<PgConnection>>,
    pub tera: Tera,
    pub reqwest: Client,
}

impl AppState {
    pub fn get_conn(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.get().unwrap()
    }

    pub fn tera_with_user(user: Option<User>) -> Context {
        let mut ctx = Context::new();

        if let Some(user) = user {
            ctx.insert("user", &user);
        }

        ctx
    }

    pub fn render_http(&self, tmpl: &'static str, ctx: &Context) -> HttpResponse {
        match self.tera.render(tmpl, &ctx) {
            Ok(s) => HttpResponse::Ok().content_type("text/html").body(s),
            Err(_) => HttpResponse::InternalServerError().into(),
        }
    }
}
