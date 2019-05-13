use crate::app::AppState;
use actix_web::{error, middleware::identity::Identity, web, Error, HttpResponse};

pub fn index(id: Identity, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut ctx = tera::Context::new();

    if let Some(url) = id.identity() {
        ctx.insert("user_url", &url);
    }

    let s = state
        .tera
        .render("index.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
