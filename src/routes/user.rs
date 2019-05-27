use crate::app::{self, PageTitle, State};
use crate::models::{user::SteamId, User};
use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use futures::Future;

pub fn user_profile(
    user_id: web::Path<SteamId>,
    user: Option<User>,
    state: web::Data<State>,
) -> impl Future<Item = HttpResponse, Error = app::Error> {
    let state2 = state.clone();

    web::block(move || {
        use crate::schema::users::dsl::*;

        users
            .filter(steam_id.eq(*user_id))
            .first::<User>(&state.get_conn())
            .map_err(app::Error::db_or_404)
    })
    .from_err()
    .and_then(move |target_user| {
        let mut ctx = State::tera_context(PageTitle::User(target_user.steam_id.to_string()), user);
        ctx.insert("target", &target_user);
        state2.render("user.html", ctx)
    })
}
