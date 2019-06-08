use crate::models::User;
use actix_web::{error::BlockingError, HttpResponse, ResponseError};
use askama::Template;
use futures::Future;
use reqwest::r#async::Client;
use std::{fs::File, io::Read, sync::Mutex};

pub type PgPool = l337::Pool<l337_postgres::PostgresConnectionManager<tokio_postgres::NoTls>>;

lazy_static! {
    static ref POOL: Mutex<PgPool> = {
        let db_cfg = std::env::var("DATABASE_URL")
            .unwrap()
            .parse()
            .expect("failed to parse db url");

        let mgr = l337_postgres::PostgresConnectionManager::new(db_cfg, tokio_postgres::NoTls);

        let pool_cfg = l337::Config {
            min_size: 1,
            max_size: 1,
        };

        Mutex::new(
            l337::Pool::new(mgr, pool_cfg)
                .wait()
                .expect("db connection error"),
        )
    };
}

pub struct State {
    pool: PgPool,
    pub reqwest: reqwest::r#async::Client,
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
    pub fn new(_db_url: &str) -> Self {
        Self {
            // DB URL is passed through env var
            pool: POOL.lock().unwrap().clone(),
            reqwest: Client::new(),
        }
    }

    pub fn get_db(&self) -> &PgPool {
        &self.pool
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
    Database(tokio_postgres::Error),
    #[fail(display = "database connection timed out")]
    DatabaseTimedOut,
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
    #[fail(display = "unknown internal server error")]
    Internal,
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
            Error::Template(e) => HttpResponse::InternalServerError().body(e.to_string()),

            #[cfg(debug_assertions)]
            x @ Error::CanceledBlock
            | x @ Error::Unauthorized
            | x @ Error::Database(_)
            | x @ Error::DatabaseTimedOut => {
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

impl From<tokio_postgres::Error> for Error {
    fn from(e: tokio_postgres::Error) -> Self {
        Error::Database(e)
    }
}

impl From<l337::Error<tokio_postgres::Error>> for Error {
    fn from(e: l337::Error<tokio_postgres::Error>) -> Self {
        match e {
            l337::Error::External(e) => Error::from(e),
            _ => Error::DatabaseTimedOut,
        }
    }
}

impl From<l337::Error<Self>> for Error {
    fn from(e: l337::Error<Self>) -> Self {
        match e {
            l337::Error::External(e) => e,
            _ => Error::DatabaseTimedOut,
        }
    }
}

impl Error {
    pub fn db_or_404(e: tokio_postgres::Error) -> Self {
        match e.code() {
            // Some(tokio_postgres::error::SqlState::NO_DATA_FOUND) => Error::NotFound,
            _ => Error::Database(e),
        }
    }
}
