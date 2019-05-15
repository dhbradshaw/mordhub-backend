use actix_web::{
    dev::Payload, http::StatusCode, middleware::identity::Identity, FromRequest, HttpRequest,
    HttpResponse, ResponseError,
};

pub struct User {
    pub id: u64,
}

#[derive(Debug, Fail)]
pub enum UserFindError {
    #[fail(display = "Database error: {}", e)]
    Db { e: diesel::result::Error },
    #[fail(display = "Identity")]
    Identity,
    #[fail(display = "Parse ID error")]
    ParseError,
    #[fail(display = "Not found")]
    NotFound,
}

impl ResponseError for UserFindError {
    fn error_response(&self) -> HttpResponse {
        match self {
            UserFindError::NotFound => HttpResponse::new(StatusCode::UNAUTHORIZED),
            _ => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

impl FromRequest for User {
    type Error = UserFindError;
    type Future = Result<Self, Self::Error>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let id = Identity::from_request(req, payload).map_err(|_| UserFindError::Identity)?;

        let steam_id_str = id.identity().ok_or(UserFindError::NotFound)?;
        let steam_id = steam_id_str
            .parse::<u64>()
            .map_err(|_| UserFindError::ParseError)?;

        // TODO: Look up in database

        Ok(User { id: steam_id })
    }
}
