use crate::app::{self, PgPool, State};
use actix_web::{dev::Payload, middleware::identity::Identity, web, FromRequest, HttpRequest};
use futures::{stream::Stream, Future, IntoFuture};
use std::str::FromStr;
use tokio_postgres::types::{FromSql, Type};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteamId(u64);

impl SteamId {
    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub fn as_i64(self) -> i64 {
        self.0 as i64
    }
}

impl FromStr for SteamId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse::<u64>()?))
    }
}

impl From<i64> for SteamId {
    fn from(i: i64) -> Self {
        Self(i as u64)
    }
}

impl From<SteamId> for i64 {
    fn from(id: SteamId) -> i64 {
        id.0 as i64
    }
}

impl<'a> FromSql<'a> for SteamId {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        i64::from_sql(ty, raw).map(SteamId::from)
    }

    fn accepts(ty: &Type) -> bool {
        i64::accepts(ty)
    }
}

impl std::fmt::Display for SteamId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.as_u64().to_string())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: i32,
    pub steam_id: SteamId,
}

impl User {
    pub fn get_by_steam_id(
        steam_id: SteamId,
        pool: &PgPool,
    ) -> impl Future<Item = User, Error = app::Error> {
        pool.connection()
            .and_then(move |mut conn| {
                conn.client
                    .prepare("SELECT id, steam_id FROM users WHERE steam_id = $1")
                    .and_then(move |statement| {
                        conn.client
                            .query(&statement, &[&steam_id.as_i64()])
                            .into_future()
                            .map(|(r, _)| r)
                            .map_err(|(e, _)| e)
                    })
                    .map_err(|e| l337::Error::External(e))
            })
            .from_err()
            .and_then(|row| match row {
                Some(row) => Ok(User {
                    id: row.get(0),
                    steam_id: row.get(1),
                }),
                None => Err(app::Error::Unauthorized),
            })
    }
}

impl FromRequest for SteamId {
    type Error = app::Error;
    type Future = Result<Self, Self::Error>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        // TODO: Should this really be an internal server error?
        let id = Identity::from_request(req, payload).map_err(|_| app::Error::Internal)?;
        id.identity()
            .ok_or(app::Error::NotFound)?
            .parse::<SteamId>()
            .map_err(|_| app::Error::NotFound)
    }
}

impl FromRequest for User {
    type Error = app::Error;
    type Future = Box<dyn Future<Item = Self, Error = app::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let state = web::Data::<State>::from_request(req, payload).expect("can't get app state");

        Box::new(
            SteamId::from_request(req, payload)
                .into_future()
                .and_then(move |steam_id| User::get_by_steam_id(steam_id, state.get_db())),
        )
    }
}
