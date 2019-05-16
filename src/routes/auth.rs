use crate::models::user::{SteamId, User};
use crate::state::AppState;
use actix_web::{error::BlockingError, middleware::identity::Identity, web, Error, HttpResponse};
use futures::stream::Concat2;
use futures::{Async, Future, Poll, Stream};
use reqwest::r#async::Decoder;
use url::Url;

const STEAM_URL: &str = "https://steamcommunity.com/openid/login";

#[derive(Serialize)]
struct OpenIdAuth {
    #[serde(rename = "openid.ns")]
    ns: &'static str,
    #[serde(rename = "openid.identity")]
    identity: &'static str,
    #[serde(rename = "openid.claimed_id")]
    claimed_id: &'static str,
    #[serde(rename = "openid.mode")]
    mode: &'static str,
    #[serde(rename = "openid.return_to")]
    return_to: String,
    #[serde(rename = "openid.realm")]
    realm: String,
}

impl OpenIdAuth {
    pub fn new(site_url: String) -> Self {
        Self {
            ns: "http://specs.openid.net/auth/2.0",
            identity: "http://specs.openid.net/auth/2.0/identifier_select",
            claimed_id: "http://specs.openid.net/auth/2.0/identifier_select",
            mode: "checkid_setup",
            return_to: format!("{}{}", site_url, "/auth/callback"),
            realm: site_url,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OpenIdVerify {
    #[serde(rename = "openid.ns")]
    ns: String,
    #[serde(rename = "openid.mode")]
    mode: String,
    #[serde(rename = "openid.op_endpoint")]
    op_endpoint: String,
    #[serde(rename = "openid.claimed_id")]
    claimed_id: String,
    #[serde(rename = "openid.identity")]
    identity: Option<String>,
    #[serde(rename = "openid.return_to")]
    return_to: String,
    #[serde(rename = "openid.response_nonce")]
    response_nonce: String,
    #[serde(rename = "openid.invalidate_handle")]
    invalidate_handle: Option<String>,
    #[serde(rename = "openid.assoc_handle")]
    assoc_handle: String,
    #[serde(rename = "openid.signed")]
    signed: String,
    #[serde(rename = "openid.sig")]
    sig: String,
}

#[derive(Deserialize, Debug)]
pub struct OpenIdVerifyResponse {
    ns: String,
    is_valid: bool,
    invalidate_handle: Option<String>,
}

#[derive(Debug)]
pub enum VerifyError {
    Reqwest(reqwest::Error),
    Deserialize,
    Invalid,
    SteamId,
    Db(diesel::result::Error),
    Interrupted,
}

pub struct UrlEncodedVerifyResponse {
    concat: Concat2<Decoder>,
    claimed_id: String,
}

impl Future for UrlEncodedVerifyResponse {
    type Item = String;
    type Error = VerifyError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let bytes = try_ready!(self.concat.poll().map_err(VerifyError::Reqwest));
        let s = std::str::from_utf8(&bytes)
            .map_err(|_| VerifyError::Deserialize)?
            .to_owned();

        // Parse ID
        let claimed_id: Self::Item = {
            let valid = s
                .split('\n')
                .map(|line| {
                    let mut pair = line.split(':');
                    (pair.next(), pair.next())
                })
                .filter_map(|(k, v)| k.and_then(|k| v.map(|v| (k, v))))
                .any(|(k, v)| k == "is_valid" && v == "true");

            if valid {
                self.claimed_id.clone()
            } else {
                return Err(VerifyError::Invalid);
            }
        };

        Ok(Async::Ready(claimed_id))
    }
}

pub fn login() -> HttpResponse {
    let openid = OpenIdAuth::new("http://localhost:3000".to_owned());

    let qs = match serde_urlencoded::to_string(&openid) {
        Ok(qs) => qs,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("failed to serialize openid 2.0 authentication parameters")
        }
    };

    let target_url = format!("{}?{}", STEAM_URL, qs);

    HttpResponse::Found()
        .header("Location", target_url)
        .finish()
}

pub fn logout(id: Identity) -> HttpResponse {
    id.forget();

    // Redirect to homepage
    HttpResponse::Found().header("Location", "/").finish()
}

pub fn callback(
    id: Identity,
    state: web::Data<AppState>,
    mut form: web::Query<OpenIdVerify>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    form.mode = "check_authentication".to_owned();

    // Send POST
    Box::new(
        state
            .reqwest
            .post(STEAM_URL)
            .form(&*form)
            .send()
            .map_err(VerifyError::Reqwest)
            .and_then(move |mut res| {
                let body = std::mem::replace(res.body_mut(), Decoder::empty());

                UrlEncodedVerifyResponse {
                    concat: body.concat2(),
                    claimed_id: form.claimed_id.clone(),
                }
            })
            .and_then(|claimed_id| {
                // Extract Steam ID
                let url = Url::parse(&claimed_id).map_err(|_| VerifyError::SteamId)?;
                let mut segments = url.path_segments().ok_or(VerifyError::SteamId)?;
                let id_segment = segments.next_back().ok_or(VerifyError::SteamId)?;

                id_segment
                    .parse::<SteamId>()
                    .map_err(|_| VerifyError::SteamId)
            })
            .and_then(|steam_id| {
                web::block(move || {
                    use crate::diesel::RunQueryDsl;
                    use crate::schema::users;

                    let new_user = User { id: steam_id };

                    diesel::insert_into(users::table)
                        .values(&new_user)
                        .on_conflict_do_nothing()
                        .execute(&state.get_conn())
                        .map(|_| steam_id)
                })
                .map_err(|e| match e {
                    BlockingError::Error(dbe) => VerifyError::Db(dbe),
                    _ => VerifyError::Interrupted,
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
            }),
    )
}
