use crate::models::User;
use crate::state::AppState;
use actix_web::{web, HttpResponse};

pub fn index(user: Option<User>, state: web::Data<AppState>) -> HttpResponse {
    let ctx = AppState::tera_with_user(user);
    state.render_http("index.html", &ctx)
}
