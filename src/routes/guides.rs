use crate::{
    app::{self, ActiveLink, State, TmplBase},
    models::User,
};
use actix_web::HttpResponse;
use askama::Template;

#[derive(Template)]
#[template(path = "guides/list.html")]
struct GuidesList {
    base: TmplBase,
}

pub fn list(user: Option<User>) -> Result<HttpResponse, app::Error> {
    State::render(GuidesList {
        base: TmplBase::new(user, ActiveLink::Guides),
    })
}
