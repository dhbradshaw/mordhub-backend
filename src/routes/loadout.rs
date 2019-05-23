use crate::app::{self, State};
use crate::models::{image::NewImage, Loadout, User};
use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use futures::Future;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateLoadout {
    name: String,
    data: String,
    cloudinary_id: String,
    cloudinary_format: String,
}

pub fn list(
    user: Option<User>,
    state: web::Data<State>,
) -> impl Future<Item = HttpResponse, Error = app::Error> {
    web::block(move || {
        use crate::schema::loadouts::dsl::*;

        loadouts
            .limit(10)
            .load::<Loadout>(&state.get_conn())
            .map(|ldts| (state, ldts))
            .map_err(app::Error::from)
    })
    .from_err()
    .and_then(|(state, ldts)| {
        let mut ctx = State::tera_with_user(user);
        ctx.insert("loadouts", &ldts);
        state.render("loadouts/list.html", ctx)
    })
}

pub fn create_get(user: Option<User>, state: web::Data<State>) -> Result<HttpResponse, app::Error> {
    if user.is_none() {
        return Err(app::Error::NotFound);
    }

    let ctx = State::tera_with_user(user);

    state.render("loadouts/create.html", ctx)
}

pub fn create_post(
    user: User,
    form: web::Form<CreateLoadout>,
    state: web::Data<State>,
) -> impl Future<Item = HttpResponse, Error = app::Error> {
    // TODO: Check CSRF token
    let cloudinary_url = format!(
        "https://res.cloudinary.com/zeta64/image/upload/{}.{}",
        &form.cloudinary_id, &form.cloudinary_format
    );

    web::block(move || {
        use crate::schema::images;

        let new_image = NewImage {
            url: cloudinary_url,
            uploader_id: user.id,
        };

        diesel::insert_into(images::table)
            .values(&new_image)
            .execute(&state.get_conn())
            .map_err(app::Error::from)
    })
    .from_err()
    /*.and_then(|image| {
        web::block(|| {
            use crate::schema::loadouts;
            let new_loadout = NewLoadout {

            };
            diesel::insert_into(loadouts
        })
    })*/
    .and_then(|_| Ok(HttpResponse::SeeOther().header("Location", "/").finish()))
}
