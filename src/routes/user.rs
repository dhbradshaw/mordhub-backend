use crate::{
    app::{self, ActiveLink, State, TmplBase},
    models::{user::SteamId, User},
};
use actix_web::{web, HttpResponse};
use askama::Template;
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
    User::get_by_steam_id(*user_id, state.get_db())
        .and_then(|target| target.ok_or(app::Error::NotFound))
        .and_then(move |target| {
            State::render(UserProfile {
                base: TmplBase::new(user, ActiveLink::None),
                target,
            })
        })
}
