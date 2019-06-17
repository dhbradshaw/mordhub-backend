use crate::{
    app::{self, State},
    models::user::SteamId,
};
use actix_web::{middleware::identity::Identity, web, HttpRequest, HttpResponse};
use futures::Future;

pub fn login(state: web::Data<State>) -> Result<HttpResponse, app::Error> {
    Ok(HttpResponse::Found()
        .header("Location", state.redirector.url().as_str())
        .finish())
}

pub fn logout(id: Identity) -> HttpResponse {
    id.forget();

    // Redirect to homepage
    HttpResponse::Found().header("Location", "/").finish()
}

pub fn callback(
    req: HttpRequest,
    id: Identity,
    state: web::Data<State>,
) -> impl Future<Item = HttpResponse, Error = app::Error> {
    steam_auth::Verifier::make_verify_request_async(&state.reqwest, req.query_string().to_owned())
        .map(SteamId::from)
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
}
