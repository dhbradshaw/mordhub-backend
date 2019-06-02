use crate::app::{self, ActiveLink, State, TmplBase};
use crate::models::User;
use actix_web::HttpResponse;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    base: TmplBase,
}

pub fn index(user: Option<User>) -> Result<HttpResponse, app::Error> {
    State::render(Index {
        base: TmplBase::new(user, ActiveLink::Home),
    })
}

#[derive(Template)]
#[template(path = "about.html")]
struct About {
    base: TmplBase,
}

pub fn about(user: Option<User>) -> Result<HttpResponse, app::Error> {
    State::render(About {
        base: TmplBase::new(user, ActiveLink::About),
    })
}
