use crate::state::AppState;
use crate::schema::users;
use diesel::sql_types::*;
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, ToSql, Output};
use std::{io, str::FromStr};
use actix_web::{
    dev::Payload, http::StatusCode, middleware::identity::Identity, FromRequest, HttpRequest,
    web::Data,
    HttpResponse, ResponseError,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromSqlRow, AsExpression)]
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

impl<DB: Backend> FromSql<BigInt, DB> for SteamId
where
    i64: FromSql<BigInt, DB>
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {    
        let v = i64::from_sql(bytes)?;
        Ok(SteamId(v as u64))
    }
}

impl<DB: Backend> ToSql<BigInt, DB> for SteamId
where
    i64: ToSql<BigInt, DB>
{
    fn to_sql<W: io::Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        self.as_i64().to_sql(out)
    }
}

#[derive(Debug, Insertable, Queryable)]
#[table_name = "users"]
pub struct User {
    pub id: SteamId,
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

impl FromRequest for User {
    type Error = UserFindError;
    type Future = Result<Self, Self::Error>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let id = Identity::from_request(req, payload).map_err(|_| UserFindError::Identity)?;

        let steam_id_str = id.identity().ok_or(UserFindError::NotFound)?;
        let steam_id = steam_id_str
            .parse::<SteamId>()
            .map_err(|_| UserFindError::ParseError)?;

        let state = Data::<AppState>::from_request(req, payload).map_err(|_| UserFindError::GetState)?;
        let conn = state.pool.get().unwrap();

        // Look up user in DB
        let user = {
            use crate::schema::users::dsl::*;
            use crate::diesel::{QueryDsl, RunQueryDsl};
            users.find(steam_id)
                .first(&conn)
                .map_err(|e| {
                    if let diesel::result::Error::NotFound = e {
                        UserFindError::NotFound
                    } else {
                        UserFindError::Db(e)
                    }
                })?
        };

        Ok(user)
    }
}