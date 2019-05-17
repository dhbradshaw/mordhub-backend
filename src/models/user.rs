use crate::schema::users;
use crate::state::AppState;
use actix_web::{
    dev::Payload, error::BlockingError, http::StatusCode, middleware::identity::Identity, web,
    FromRequest, HttpRequest, HttpResponse, ResponseError,
};
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::*;
use futures::{Future, IntoFuture};
use std::{io::Write, str::FromStr};

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromSqlRow, AsExpression, Deserialize)]
#[sql_type = "BigInt"]
pub struct SteamId(u64);

impl SteamId {
    pub fn as_u64(self) -> u64 {
        self.0
    }

    fn as_i64(self) -> i64 {
        self.0 as i64
    }
}

impl FromStr for SteamId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse::<u64>()?))
    }
}

impl std::fmt::Display for SteamId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.as_u64().to_string())
    }
}

impl<DB: Backend> FromSql<BigInt, DB> for SteamId
where
    i64: FromSql<BigInt, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        let v = i64::from_sql(bytes)?;
        Ok(SteamId(v as u64))
    }
}

impl<DB: Backend> ToSql<BigInt, DB> for SteamId
where
    i64: ToSql<BigInt, DB>,
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        self.as_i64().to_sql(out)
    }
}

#[derive(Debug, Insertable, Queryable)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub steam_id: SteamId,
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub steam_id: SteamId,
}

#[derive(Debug, Fail)]
pub enum UserFindError {
    #[fail(display = "Database error: {}", _0)]
    Db(diesel::result::Error),
    #[fail(display = "Identity error")]
    Identity,
    #[fail(display = "Parse ID error")]
    ParseError,
    #[fail(display = "State error")]
    GetState,
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

impl FromRequest for SteamId {
    type Error = UserFindError;
    type Future = Result<Self, Self::Error>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let id = Identity::from_request(req, payload).map_err(|_| UserFindError::Identity)?;
        id.identity()
            .ok_or(UserFindError::NotFound)?
            .parse::<SteamId>()
            .map_err(|_| UserFindError::ParseError)
    }
}

impl FromRequest for User {
    type Error = BlockingError<UserFindError>;
    type Future = Box<dyn Future<Item = Self, Error = BlockingError<UserFindError>>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let state = web::Data::<AppState>::from_request(req, payload).expect("can't get app state");

        Box::new(
            SteamId::from_request(req, payload)
                .map_err(BlockingError::Error)
                .into_future()
                .and_then(move |s_id| {
                    web::block(move || {
                        let user = {
                            use crate::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
                            use crate::schema::users::dsl::*;
                            users
                                .filter(steam_id.eq(s_id))
                                .first(&state.get_conn())
                                .map_err(|e| {
                                    if let diesel::result::Error::NotFound = e {
                                        UserFindError::NotFound
                                    } else {
                                        UserFindError::Db(e)
                                    }
                                })?
                        };

                        Ok(user)
                    })
                }),
        )
    }
}
