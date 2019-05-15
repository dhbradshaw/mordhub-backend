use crate::models::User;
use crate::state::AppState;
use actix_web::{error, middleware::identity::Identity, web, Error, HttpResponse};

pub fn index(user: Option<User>, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut ctx = tera::Context::new();

    if let Some(user) = user {
        ctx.insert("user_id", &user.id.as_u64());
    }

    let s = state
        .tera
        .render("index.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
