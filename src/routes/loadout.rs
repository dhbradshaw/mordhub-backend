use crate::{
    app::{self, ActiveLink, State, TmplBase},
    models::{Image, LoadoutMultiple, LoadoutSingle, User},
};
use actix_web::{web, HttpResponse};
use askama::Template;
use futures::{stream::Stream, Future};

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

    LoadoutMultiple::query(user2, state.get_db()).and_then(move |loadouts| {
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

    state
        .get_db()
        .connection()
        .from_err()
        .and_then(move |mut conn_ptr| {
            let conn = &mut *conn_ptr;
            conn.client
                .query(
                    &conn.queries.create_loadout,
                    &[&user_id, &form.name, &form.data],
                )
                .into_future()
                .map(|(r, _)| (conn_ptr, r))
                .map_err(|(e, _)| app::Error::from(e))
        })
        .and_then(|(conn, row)| match row {
            Some(row) => Ok((conn, row.get::<_, i32>(0))),
            None => Err(app::Error::DbNothingReturned),
        })
        .and_then(move |(mut conn, loadout_id)| {
            // TODO: Handle position when multiple images uploaded
            let conn = &mut *conn;
            conn.client
                .query(
                    &conn.queries.create_image,
                    &[&cloudinary_url, &loadout_id, &0i32],
                )
                .into_future()
                .map_err(|(e, _)| e)
                .map(move |_| loadout_id)
                .from_err()
        })
        .and_then(|loadout_id| {
            Ok(HttpResponse::SeeOther()
                .header("Location", format!("/loadouts/{}", loadout_id))
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
    state
        .get_db()
        .connection()
        .from_err()
        .and_then(move |mut conn| {
            let loadout_future = LoadoutSingle::query(*ld_id as i32, user.clone(), &mut conn)
                .and_then(|ldt| ldt.ok_or(app::Error::NotFound));

            let images_future = Image::query(*ld_id as i32, &mut conn);

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
        })
}
