use crate::models::User;
use actix_web::HttpResponse;
use askama::Template;
use reqwest::r#async::Client;

pub use crate::error::Error;

pub type PgPool = l337::Pool<l337_postgres::PostgresConnectionManager<tokio_postgres::NoTls>>;

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
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            reqwest: Client::new(),
        }
    }

    pub fn get_db(&self) -> &PgPool {
        &self.pool
    }

    pub fn render<T: Template>(ctx: T) -> Result<HttpResponse, Error> {
        match ctx.render() {
            Ok(s) => Ok(HttpResponse::Ok().content_type("text/html").body(s)),
            Err(e) => Err(Error::Template(e)),
        }
    }
}
