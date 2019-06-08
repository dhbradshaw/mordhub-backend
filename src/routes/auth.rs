use crate::{
    app::{self, State},
    models::user::SteamId,
};
use actix_web::{middleware::identity::Identity, web, Error, HttpResponse};
use futures::{Future, Stream};
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
    Db(tokio_postgres::Error),
    DbTimeout,
}

impl From<tokio_postgres::Error> for VerifyError {
    fn from(e: tokio_postgres::Error) -> Self {
        VerifyError::Db(e)
    }
}

pub fn login() -> HttpResponse {
    let openid = OpenIdAuth::new(std::env::var("SITE_URL").expect("SITE_URL is not set"));

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
    state: web::Data<State>,
    mut form: web::Query<OpenIdVerify>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    form.mode = "check_authentication".to_owned();

    // TODO: Really should clean up the error handling in here
    // Send POST
    state
        .reqwest
        .post(STEAM_URL)
        .form(&*form)
        .send()
        .map_err(VerifyError::Reqwest)
        .and_then(|res| res.into_body().concat2().map_err(VerifyError::Reqwest))
        .and_then(move |body| {
            let s = std::str::from_utf8(&body)
                .map_err(|_| VerifyError::Deserialize)?
                .to_owned();

            // Parse ID and return it
            let valid = s
                .split('\n')
                .map(|line| {
                    let mut pair = line.split(':');
                    (pair.next(), pair.next())
                })
                .filter_map(|(k, v)| k.and_then(|k| v.map(|v| (k, v))))
                .any(|(k, v)| k == "is_valid" && v == "true");

            if valid {
                Ok(form.claimed_id.clone())
            } else {
                Err(VerifyError::Invalid)
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
        .and_then(move |steam_id| {
            state
                .get_db()
                .connection()
                .and_then(move |mut conn| {
                    conn.client
                        .prepare("INSERT INTO users (steam_id) VALUES ($1) ON CONFLICT DO NOTHING")
                        .and_then(move |statement| {
                            conn.client
                                .execute(&statement, &[&steam_id.as_i64()])
                                .map(move |_| steam_id)
                        })
                        .map_err(|e| l337::Error::External(e))
                })
                .map_err(|e| match e {
                    l337::Error::External(e) => VerifyError::Db(e),
                    _ => VerifyError::DbTimeout,
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
