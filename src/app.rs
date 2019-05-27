use crate::models::User;
use actix_web::{error::BlockingError, HttpResponse, ResponseError};
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};
use reqwest::r#async::Client;
use std::fs::File;
use std::io::Read;
use tera::{Context, Tera, Value};

pub struct State {
    pub pool: Pool<ConnectionManager<PgConnection>>,
    pub tera: Tera,
    pub reqwest: Client,
}

#[derive(Debug, Clone)]
pub enum PageTitle {
    Home,
    LoadoutList,
    LoadoutCreate,
    LoadoutSingle(String),
    User(String),
    About,
}

impl State {
    pub fn get_conn(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.get().unwrap()
    }

    pub fn tera_context(page: PageTitle, user: Option<User>) -> Context {
        let mut ctx = Context::new();

        if let Some(user) = user {
            ctx.insert("user", &user);
        }

        let (page_type, page_title) = match page {
            PageTitle::Home => ("Home", "Home".to_string()),
            PageTitle::LoadoutList => ("Loadouts", "Loadouts".to_string()),
            PageTitle::LoadoutCreate => ("Loadouts", "Create Loadout".to_string()),
            PageTitle::LoadoutSingle(s) => ("Loadouts", s),
            PageTitle::User(s) => ("User", s),
            PageTitle::About => ("About", "About".to_string()),
        };

        ctx.insert("page_type", page_type);
        ctx.insert("page_title", &page_title);

        ctx
    }

    pub fn render(&self, tmpl: &'static str, ctx: Context) -> Result<HttpResponse, Error> {
        match self.tera.render(tmpl, ctx) {
            Ok(s) => Ok(HttpResponse::Ok().content_type("text/html").body(s)),
            Err(e) => Err(crate::app::Error::Template(format!("{:?}", e))),
        }
    }
}

pub fn tera_streq(value: Option<&Value>, args: &[Value]) -> Result<bool, tera::Error> {
    let value = value
        .and_then(Value::as_str)
        .expect("streq test provided with no value");
    let other = args
        .first()
        .and_then(Value::as_str)
        .expect("streq test provided with no argument");
    Ok(value == other)
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "database error: {}", _0)]
    Database(diesel::result::Error),
    #[fail(display = "template error: {}", _0)]
    Template(String),
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
