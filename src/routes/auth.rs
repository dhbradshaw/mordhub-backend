use crate::{app::{self, State}, models::user::SteamId};
use actix_web::{middleware::identity::Identity, web, Error, HttpResponse};
use futures::Future;

pub fn login() -> Result<HttpResponse, app::Error> {
    let site_url = std::env::var("SITE_URL").expect("SITE_URL is not set");

    // TODO: Put this in app::State
    let target_url = steam_auth::get_login_url(site_url, "/auth/callback")
        .map_err(app::Error::SteamAuth)?;

    Ok(HttpResponse::Found()
        .header("Location", target_url.as_str())
        .finish())
}

pub fn logout(id: Identity) -> HttpResponse {
    id.forget();

    // Redirect to homepage
    HttpResponse::Found().header("Location", "/").finish()
}

pub fn callback(
    id: Identity,
    state: web::Data<State>,
    form: web::Query<steam_auth::SteamAuthResponse>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    steam_auth::verify_response_async(&state.reqwest, form.into_inner())
        .map(|steam_id| SteamId::from(steam_id))
        .map_err(app::Error::SteamAuth)
        .and_then(move |steam_id| {
            state
                .get_db()
                .connection()
                .from_err()
                .and_then(move |mut conn| {
                    let conn = &mut *conn;
                    conn.client
                        .execute(&conn.queries.post_login_insert_user, &[&steam_id.as_i64()])
                        .map(move |_| steam_id)
                        .from_err()
                })
        })
        .and_then(move |steam_id| {
            debug!("Verified user {}", steam_id);
            id.remember(steam_id.to_string());
            Ok(HttpResponse::Found().header("Location", "/").finish())
        })
        .map_err(|e| {
            debug!("Verify error: {:?}", e);
            HttpResponse::Unauthorized()
                .body("authentication failed")
                .into()
        })
}
