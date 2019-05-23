use crate::app::{self, State};
use crate::models::user::SteamId;
use actix_web::web;

pub fn user_profile(
    id: Option<web::Path<SteamId>>,
    _state: web::Data<State>,
) -> Result<String, app::Error> {
    if let Some(id) = id {
        Ok(format!("Showing info for {}", id))
    } else {
        Err(app::Error::NotFound)
    }
}
