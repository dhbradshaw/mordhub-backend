use crate::models::{Loadout, User};
use crate::state::AppState;
use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use futures::Future;

pub fn list(
    user: Option<User>,
    state: web::Data<AppState>,
) -> impl Future<Item = HttpResponse, Error = ()> {
    web::block(move || {
        use crate::schema::loadouts::dsl::*;

        loadouts
            .limit(10)
            .load::<Loadout>(&state.get_conn())
            .map(|ldts| (state, ldts))
    })
    .map_err(|_| ())
    .and_then(|(state, ldts)| {
        let mut ctx = AppState::tera_with_user(user);
        ctx.insert("loadouts", &ldts);
        Ok(state.render_http("loadouts/list.html", &ctx))
    })
    .or_else(|_| Ok(HttpResponse::InternalServerError().into()))
}

pub fn create(user: Option<User>, state: web::Data<AppState>) -> HttpResponse {
    if user.is_none() {
        return HttpResponse::Found()
            .header("Location", "/auth/login")
            .finish();
    }

    let ctx = AppState::tera_with_user(user);

    state.render_http("loadouts/create.html", &ctx)
}
