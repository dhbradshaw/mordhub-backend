use crate::{
    app::{self, ActiveLink, State, TmplBase},
    models::{Image, LoadoutMultiple, LoadoutSingle, NewImage, NewLoadout, User},
};
use actix_web::{web, HttpResponse};
use askama::Template;
use diesel::prelude::*;
use futures::Future;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateLoadout {
    name: String,
    data: String,
    cloudinary_id: String,
    cloudinary_format: String,
}

#[derive(Template)]
#[template(path = "loadouts/list.html")]
struct LoadoutList {
    base: TmplBase,
    loadouts: Vec<LoadoutMultiple>,
}

pub fn list(
    user: Option<User>,
    state: web::Data<State>,
) -> impl Future<Item = HttpResponse, Error = app::Error> {
    let user2 = user.clone();

    web::block(move || LoadoutMultiple::query(user2, &state.get_conn()).map_err(app::Error::from))
        .from_err()
        .and_then(move |loadouts| {
            State::render(LoadoutList {
                base: TmplBase::new(user, ActiveLink::Loadouts),
                loadouts,
            })
        })
}

#[derive(Template)]
#[template(path = "loadouts/create.html")]
struct LoadoutCreate {
    base: TmplBase,
}

pub fn create_get(user: Option<User>) -> Result<HttpResponse, app::Error> {
    if user.is_none() {
        return Err(app::Error::RedirectToLogin);
    }

    State::render(LoadoutCreate {
        base: TmplBase::new(user, ActiveLink::Loadouts),
    })
}

pub fn create_post(
    user: User,
    form: web::Form<CreateLoadout>,
    state: web::Data<State>,
) -> impl Future<Item = HttpResponse, Error = app::Error> {
    // TODO: Check CSRF token
    // TODO: Sanitize these inputs
    let cloudinary_url = format!(
        "https://res.cloudinary.com/zeta64/image/upload/{}.{}",
        &form.cloudinary_id, &form.cloudinary_format
    );

    let user_id = user.id;
    let state_2 = state.clone();

    web::block(move || {
        use crate::schema::loadouts;

        let new_loadout = NewLoadout {
            user_id,
            name: form.name.clone(),
            data: form.data.clone(),
        };

        diesel::insert_into(loadouts::table)
            .values(&new_loadout)
            .returning(loadouts::id)
            .get_result::<i32>(&state.get_conn())
            .map_err(app::Error::from)
    })
    .from_err()
    .and_then(move |loadout_id| {
        web::block(move || {
            use crate::schema::images;

            let new_image = NewImage {
                url: cloudinary_url,
                loadout_id,
                position: 0,
            };

            diesel::insert_into(images::table)
                .values(&new_image)
                .execute(&state_2.clone().get_conn())
                .map_err(app::Error::from)
                .map(|_| loadout_id)
        })
    })
    .from_err()
    .and_then(|id| {
        Ok(HttpResponse::SeeOther()
            .header("Location", format!("/loadouts/{}", id))
            .finish())
    })
}

#[derive(Template)]
#[template(path = "loadouts/single.html")]
struct LoadoutSingleTmpl {
    base: TmplBase,
    loadout: LoadoutSingle,
    images: Vec<Image>,
}

pub fn single(
    ld_id: web::Path<u32>,
    user: Option<User>,
    state: web::Data<State>,
) -> impl Future<Item = HttpResponse, Error = app::Error> {
    let state2 = state.clone();
    let ld_id = *ld_id;
    let user2 = user.clone();

    let loadout_future = web::block(move || {
        LoadoutSingle::query(ld_id as i32, user2, &state.get_conn()).map_err(app::Error::db_or_404)
    })
    .from_err();

    let images_future = web::block(move || {
        use crate::schema::images::dsl::*;

        images
            .filter(loadout_id.eq(ld_id as i32))
            .order_by(position)
            .load::<Image>(&state2.get_conn())
            .map_err(app::Error::from)
    })
    .from_err();

    // Run queries in parallel
    images_future.join(loadout_future).and_then(
        move |(images, loadout): (Vec<Image>, LoadoutSingle)| {
            State::render(LoadoutSingleTmpl {
                base: TmplBase::new(user, ActiveLink::Loadouts),
                loadout,
                images,
            })
        },
    )
}
