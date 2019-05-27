use crate::app::{self, PageTitle, State};
use crate::models::User;
use actix_web::{web, HttpResponse};

pub fn index(user: Option<User>, state: web::Data<State>) -> Result<HttpResponse, app::Error> {
    let ctx = State::tera_context(PageTitle::Home, user);
    state.render("index.html", ctx)
}

pub fn about(user: Option<User>, state: web::Data<State>) -> Result<HttpResponse, app::Error> {
    let ctx = State::tera_context(PageTitle::About, user);
    state.render("about.html", ctx)
}
