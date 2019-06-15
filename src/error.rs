use crate::files;
use actix_web::{error::BlockingError, HttpResponse, ResponseError};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "database error: {}", _0)]
    Database(tokio_postgres::Error),
    #[fail(display = "template error: {}", _0)]
    Template(askama::Error),
    #[fail(display = "canceled block")]
    CanceledBlock,
    #[fail(display = "404 not found")]
    NotFound,
    #[fail(display = "unauthorized")]
    Unauthorized,
    #[fail(display = "unknown internal error")]
    Internal,
    #[fail(display = "nothing returned from database")]
    DbNothingReturned,
    #[fail(display = "unauthorized - redirecting to login")]
    RedirectToLogin,
    #[fail(display = "failed to authenticate with steam: {}", _0)]
    SteamAuth(steam_auth::Error),
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        #[allow(unreachable_patterns)]
        match self {
            Error::NotFound => HttpResponse::NotFound()
                .content_type("text/html")
                .body(files::read("static/404.html")),

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

impl From<l337::Error<tokio_postgres::Error>> for Error {
    fn from(e: l337::Error<tokio_postgres::Error>) -> Self {
        match e {
            l337::Error::Internal(_) => Error::Internal,
            l337::Error::External(e) => Error::Database(e),
        }
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(e: tokio_postgres::Error) -> Self {
        Error::Database(e)
    }
}

impl Error {
    pub fn db_or_404(e: tokio_postgres::Error) -> Self {
        match e {
            // TODO: Work out what the error code is
            // tokio_postgres::Error::NotFound => Error::NotFound,
            _ => Error::Database(e),
        }
    }
}
