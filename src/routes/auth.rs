use actix_web::{web, Responder};
use crate::app::AppState;

pub fn login(_state: web::Data<AppState>) -> impl Responder {
    "hello world".to_owned()
}
