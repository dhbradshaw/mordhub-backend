use crate::app::{self, State};
use crate::models::{image::NewImage, loadout::NewLoadout, Image, Like, Loadout, User};
use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use futures::{future::Either, Future, IntoFuture};

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
        return Err(app::Error::RedirectToLogin);
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
            .get_result::<Loadout>(&state.get_conn())
            .map_err(app::Error::from)
    })
    .from_err()
    .and_then(move |loadout| {
        web::block(move || {
            use crate::schema::images;

            let new_image = NewImage {
                url: cloudinary_url,
                loadout_id: loadout.id,
                position: 0,
            };

            diesel::insert_into(images::table)
                .values(&new_image)
                .execute(&state_2.clone().get_conn())
                .map_err(app::Error::from)
                .map(|_| loadout.id)
        })
    })
    .from_err()
    .and_then(|id| {
        Ok(HttpResponse::SeeOther()
            .header("Location", format!("/loadouts/{}", id))
            .finish())
    })
}

pub fn single(
    ld_id: web::Path<u32>,
    user: Option<User>,
    state: web::Data<State>,
) -> impl Future<Item = HttpResponse, Error = app::Error> {
    let state2 = state.clone();
    let state3 = state.clone();
    let state4 = state.clone();
    let state5 = state.clone();
    let ld_id = *ld_id;

    let loadout_future = web::block(move || {
        use crate::schema::loadouts::dsl::*;

        loadouts
            .find(ld_id as i32)
            .first(&state.get_conn())
            .map_err(app::Error::db_or_404)
    })
    .from_err();

    let like_count_future = web::block(move || {
        use crate::schema::likes::dsl::*;

        likes
            .filter(loadout_id.eq(ld_id as i32))
            .count()
            .get_result(&state2.get_conn())
            .map_err(app::Error::from)
    })
    .from_err();

    let images_future = web::block(move || {
        use crate::schema::images::dsl::*;

        images
            .filter(loadout_id.eq(ld_id as i32))
            .order_by(position)
            .load::<Image>(&state3.get_conn())
            .map_err(app::Error::from)
    })
    .from_err();

    let has_liked_future = if let Some(user) = user.as_ref().cloned() {
        // If they're logged in, check to see if they already liked this
        Either::A(
            web::block(move || {
                use crate::schema::likes::dsl::*;

                let res = likes
                    .filter(user_id.eq(user.id))
                    .filter(loadout_id.eq(ld_id as i32))
                    .first::<Like>(&state4.get_conn());

                match res {
                    Ok(_) => Ok(true),
                    Err(diesel::result::Error::NotFound) => Ok(false),
                    Err(e) => Err(app::Error::from(e)),
                }
            })
            .from_err(),
        )
    } else {
        // Otherwise, they didn't already like this
        Either::B(Ok(false).into_future())
    };

    // Run queries in parallel
    // TODO: Make 404 reliable
    images_future
        .join4(loadout_future, like_count_future, has_liked_future)
        .and_then(
            move |(images, loadout, like_count, has_liked): (Vec<Image>, Loadout, i64, bool)| {
                let mut ctx = State::tera_with_user(user);
                ctx.insert("loadout", &loadout);
                ctx.insert("images", &images);
                ctx.insert("like_count", &like_count);
                ctx.insert("has_liked", &has_liked);
                state5.render("loadouts/single.html", ctx)
            },
        )
}
