use crate::{
    app::{self, ActiveLink, State, TmplBase},
    models::{user::SteamId, User},
};
use actix_web::{web, HttpResponse};
use askama::Template;
use diesel::prelude::*;
use futures::Future;

#[derive(Template)]
#[template(path = "user.html")]
struct UserProfile {
    base: TmplBase,
    target: User,
}

pub fn user_profile(
    user_id: web::Path<SteamId>,
    user: Option<User>,
    state: web::Data<State>,
) -> impl Future<Item = HttpResponse, Error = app::Error> {
    web::block(move || {
        use crate::schema::users::dsl::*;

        users
            .filter(steam_id.eq(*user_id))
            .first::<User>(&state.get_conn())
            .map_err(app::Error::db_or_404)
    })
    .from_err()
    .and_then(move |target| {
        State::render(UserProfile {
            base: TmplBase::new(user, ActiveLink::None),
            target,
        })
    })
}
