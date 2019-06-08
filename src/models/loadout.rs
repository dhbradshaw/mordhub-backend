use crate::{
    app::{self, PgPool},
    models::{user::SteamId, User},
};
use chrono::naive::NaiveDateTime;
use futures::{
    future::{self, Either, Future},
    stream::Stream,
};

#[derive(Debug, Clone, Serialize)]
pub struct LoadoutSingle {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub data: String,
    pub created_at: NaiveDateTime,
    pub like_count: i64,
    pub has_liked: bool, // Whether the current user has already liked this loadout
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadoutMultiple {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub data: String,
    pub created_at: NaiveDateTime,
    pub like_count: i64,
    pub has_liked: bool, // Whether the current user has already liked this loadout
    pub main_image_url: String,
    pub user_steam_id: SteamId,
}

#[derive(Debug, Clone)]
pub struct NewLoadout {
    pub user_id: i32,
    pub name: String,
    pub data: String,
}

impl LoadoutMultiple {
    pub fn query(user: Option<User>, conn: &PgPool) -> Result<Vec<Self>, tokio_postgres::Error> {
        Ok(vec![])
        // if let Some(user) = user {
        //     diesel::sql_query(
        //         "SELECT loadouts.*, \
        //             (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id)
        // as like_count, \             EXISTS (SELECT user_id FROM likes WHERE
        // user_id = $1) AS has_liked, \             (SELECT steam_id FROM users
        // WHERE users.id = loadouts.user_id) as user_steam_id, \
        // (SELECT url FROM images WHERE images.loadout_id = loadouts.id AND
        // images.position = 0) as main_image_url \         FROM loadouts"
        //     )
        //         .bind::<Integer, _>(user.id)
        //         .get_results(conn)
        // } else {
        //     diesel::sql_query(
        //         "SELECT loadouts.*, \
        //             (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id)
        // as like_count, \             (SELECT steam_id FROM users WHERE
        // users.id = loadouts.user_id) as user_steam_id, \             (SELECT
        // url FROM images WHERE images.loadout_id = loadouts.id AND images.position =
        // 0) as main_image_url \         FROM loadouts"
        //     )
        //         .get_results::<Self>(conn)
        //         .map(|res| res.into_iter().map(|mut r| { r.has_liked = false; r
        // }).collect()) }
    }
}

impl LoadoutSingle {
    pub fn query(
        id: i32,
        user: Option<User>,
        pool: &PgPool,
    ) -> impl Future<Item = Self, Error = app::Error> {
        pool.connection().and_then(|mut conn| {
            if let Some(user) = user {
                Either::A(conn.client.prepare(
                    "SELECT (id, user_id, name, data, created_at), \
                    (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id) as like_count, \
                    EXISTS (SELECT user_id FROM likes WHERE user_id = $1) AS has_liked \
                    FROM loadouts \
                    WHERE loadouts.id = $2"
                ).map_err(|e| l337::Error::External(e))
                .map(move |row| (conn, row, user.id)))
            } else {
                Either::B(conn.client.prepare(
                    "SELECT loadouts.*, \
                    (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id) as like_count \
                    FROM loadouts \
                    WHERE loadouts.id = $1"
                ).map_err(|e| l337::Error::External(e))
                .map(|row| (conn, row, 0)))
            }
        })
        .from_err()
        .and_then(move |(mut conn, statement, user_id)| {
            conn.client.query(&statement, &[&user_id, &id])
                .into_future()
                .map(|(r, _)| r)
                .map_err(|(e, _)| e)
                .from_err()
        })
        .and_then(|row| match row {
            Some(row) => Ok(LoadoutSingle {
                id: row.get(0),
                user_id: row.get(1),
                name: row.get(2),
                data: row.get(3),
                created_at: row.get(4),
                like_count: row.get(5),
                has_liked: row.get(6)
            }),
            None => Err(app::Error::NotFound)
        })
    }
}
