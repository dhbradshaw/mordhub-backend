use crate::app::AppState;
use actix_web::{error, middleware::identity::Identity, web, Error, HttpResponse};

pub fn index(id: Identity, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut ctx = tera::Context::new();
    ctx.insert(
        "user_url",
        id.identity()
            .as_ref()
            .map(|s| &**s)
            .unwrap_or("(not logged in)"),
    );

    let s = state
        .tera
        .render("index.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
