use crate::futures::Future;
use crate::models::Loadout;
use crate::state::AppState;
use actix_web::{error, web, HttpResponse};
use diesel::prelude::*;

pub fn list(state: web::Data<AppState>) -> impl Future<Item = HttpResponse, Error = ()> {
    web::block(move || {
        use crate::schema::loadouts::dsl::*;

        loadouts
            .limit(10)
            .load::<Loadout>(&state.get_conn())
            .map(|ldts| (state, ldts))
    })
    .map_err(|_| ())
    .and_then(|(state, ldts)| {
        let mut ctx = tera::Context::new();

        ctx.insert("loadouts", &ldts);

        state
            .tera
            .render("loadouts/list.html", &ctx)
            .map_err(|_| ())
    })
    .then(|string| match string {
        Ok(s) => Ok(HttpResponse::Ok().content_type("text/html").body(s)),
        Err(_) => Ok(HttpResponse::InternalServerError().into()),
    })
}
