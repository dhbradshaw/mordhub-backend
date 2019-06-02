use crate::models::User;
use actix_web::{error::BlockingError, HttpResponse, ResponseError};
use askama::Template;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};
use reqwest::r#async::Client;
use std::fs::File;
use std::io::Read;

pub struct State {
    pub pool: Pool<ConnectionManager<PgConnection>>,
    pub reqwest: Client,
}

#[derive(Debug, Clone)]
pub enum ActiveLink {
    Home,
    Loadouts,
    Guides,
    About,
    None,
}

pub struct TmplBase {
    pub user: Option<User>,
    pub active_link: ActiveLink,
}

impl TmplBase {
    pub fn new(user: Option<User>, al: ActiveLink) -> Self {
        Self {
            user,
            active_link: al,
        }
    }
}

impl State {
    pub fn get_conn(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.get().unwrap()
    }

    pub fn render<T: Template>(ctx: T) -> Result<HttpResponse, Error> {
        match ctx.render() {
            Ok(s) => Ok(HttpResponse::Ok().content_type("text/html").body(s)),
            Err(e) => Err(crate::app::Error::Template(e)),
        }
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "database error: {}", _0)]
    Database(diesel::result::Error),
    #[fail(display = "template error: {}", _0)]
    Template(askama::Error),
    #[fail(display = "canceled block")]
    CanceledBlock,
    #[fail(display = "404 not found")]
    NotFound,
    #[fail(display = "unauthorized")]
    Unauthorized,
    #[fail(display = "unauthorized - redirecting to login")]
    RedirectToLogin,
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        #[allow(unreachable_patterns)]
        match self {
            Error::NotFound => {
                // TODO: Pre-cache this in memory
                let f = File::open("static/404.html");
                f.and_then(|mut f| {
                    let mut s = String::new();
                    f.read_to_string(&mut s).map(|_| s)
                })
                .and_then(|s| Ok(HttpResponse::NotFound().content_type("text/html").body(s)))
                .unwrap_or_else(|_| HttpResponse::NotFound().into())
            }

            Error::RedirectToLogin => HttpResponse::Found()
                .header("Location", "/auth/login")
                .finish(),

            #[cfg(debug_assertions)]
            x @ Error::Database(_) => HttpResponse::InternalServerError().body(x.to_string()),

            #[cfg(debug_assertions)]
            Error::Template(e) => HttpResponse::InternalServerError().body(e.to_string()),

            #[cfg(debug_assertions)]
            x @ Error::CanceledBlock | x @ Error::Unauthorized => {
                HttpResponse::InternalServerError().body(x.to_string())
            }

            // TODO: Render a nice HTML file instead in release
            _ => HttpResponse::InternalServerError().body("Unknown internal server error"),
        }
    }

    fn render_response(&self) -> HttpResponse {
        self.error_response()
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

impl Error {
    pub fn db_or_404(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => Error::NotFound,
            _ => Error::Database(e),
        }
    }
}
