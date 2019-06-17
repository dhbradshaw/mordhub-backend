use crate::{
    app::{self, State},
    models::user::SteamId,
};
use actix_web::{middleware::identity::Identity, web, HttpResponse};
use futures::{
    future::{self, Either},
    Future,
};

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
    id: Identity,
    state: web::Data<State>,
    form: web::Query<steam_auth::SteamLoginData>,
) -> impl Future<Item = HttpResponse, Error = app::Error> {
    let result =
        steam_auth::Verifier::from_parsed(form.into_inner()).map_err(app::Error::SteamAuth);

    let (req, verifier) = match result {
        Ok((r, v)) => (r, v),
        Err(e) => return Either::A(future::err(e)),
    };

    let (parts, body) = req.into_parts();
    let mut out_req = state.client.request(parts.method, parts.uri);

    for (k, v) in &parts.headers {
        out_req = out_req.set_header(k, v.as_bytes());
    }

    Either::B(
        out_req
            .send_body(body)
            .map_err(|e| {
                error!("error sending steam login request: {}", e);
                app::Error::SendRequestError
            })
            .and_then(|mut res| res.body().map_err(|_| app::Error::SendRequestError))
            .and_then(|body| {
                std::str::from_utf8(body.as_ref())
                    .map_err(|_| app::Error::SendRequestError)
                    .map(str::to_owned)
            })
            .and_then(|body| {
                verifier
                    .verify_response(body)
                    .map_err(app::Error::SteamAuth)
            })
            .map(SteamId::from)
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
            }),
    )
}
