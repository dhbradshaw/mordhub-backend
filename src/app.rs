use crate::models::User;
use actix_web::{error::BlockingError, HttpResponse, Responder, ResponseError};
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};
use reqwest::r#async::Client;
use std::fs::File;
use std::io::Read;
use tera::{Context, Tera};

pub struct State {
    pub pool: Pool<ConnectionManager<PgConnection>>,
    pub tera: Tera,
    pub reqwest: Client,
}

impl State {
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

    pub fn render(&self, tmpl: &'static str, ctx: Context) -> Result<HttpResponse, Error> {
        match self.tera.render(tmpl, ctx) {
            Ok(s) => Ok(HttpResponse::Ok().content_type("text/html").body(s)),
            Err(_) => Err(crate::app::Error::Template),
        }
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "database error: {}", _0)]
    Database(diesel::result::Error),
    #[fail(display = "template error")]
    Template,
    #[fail(display = "canceled block")]
    CanceledBlock,
    #[fail(display = "404 not found")]
    NotFound,
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::NotFound => {
                let f = File::open("static/404.html");
                f.and_then(|mut f| {
                    let mut s = String::new();
                    f.read_to_string(&mut s).map(|_| s)
                })
                .and_then(|s| Ok(HttpResponse::NotFound().content_type("text/html").body(s)))
                .unwrap_or_else(|_| HttpResponse::InternalServerError().into())
            }
            #[cfg(debug_assertions)]
            Error::Database(e) => HttpResponse::InternalServerError().body(e.to_string()),
            _ => HttpResponse::InternalServerError().into(),
        }
    }
}

impl From<Error> for HttpResponse {
    fn from(e: Error) -> HttpResponse {
        e.error_response()
    }
}

impl From<BlockingError<Error>> for Error {
    fn from(e: BlockingError<Error>) -> Self {
        match e {
            BlockingError::Error(e) => e,
            BlockingError::Canceled => Error::CanceledBlock,
        }
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Error::Database(e)
    }
}
