use crate::models::user::{SteamId, User};
use crate::state::AppState;
use crate::Or404;
use actix_web::{web, Either, Error, HttpResponse};

pub fn user_profile(id: Option<web::Path<SteamId>>, _state: web::Data<AppState>) -> Or404<String> {
    if let Some(id) = id {
        Either::A(format!("Showing info for {}", id))
    } else {
        Either::B(crate::handle_404())
    }
}
