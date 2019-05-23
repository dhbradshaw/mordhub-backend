use crate::app::{self, State};
use crate::models::User;
use actix_web::{web, HttpResponse};

pub fn index(user: Option<User>, state: web::Data<State>) -> Result<HttpResponse, app::Error> {
    let ctx = State::tera_with_user(user);
    state.render("index.html", ctx)
}
