use crate::app::AppState;
use actix_web::{error, web, Error, HttpResponse};

pub fn index(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let ctx = tera::Context::new();

    let s = state
        .tera
        .render("index.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
