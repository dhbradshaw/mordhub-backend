use crate::models::user::{SteamId, User};
use crate::state::AppState;
use actix_web::{web, Error, HttpResponse, Responder};

pub fn user_profile(id: web::Path<SteamId>, _state: web::Data<AppState>) -> impl Responder {
    format!("Showing info for {}", id)
}
