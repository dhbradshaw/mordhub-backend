use crate::{
    app::{self, PgPool},
    models::{user::SteamId, User},
};
use chrono::naive::NaiveDateTime;
use futures::{future::Future, stream::Stream};

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
        loadout_id: i32,
        user: Option<User>,
        pool: &PgPool,
    ) -> impl Future<Item = Option<Self>, Error = app::Error> {
        let with_user = user.is_some();

        pool.connection().and_then(move |mut conn| {
            // f3d5pzxulmlbpanpf5sc
            let query_with_user = "SELECT id, user_id, name, data, created_at, \
                    (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id) as like_count, \
                    EXISTS (SELECT user_id FROM likes WHERE user_id = $1) AS has_liked \
                    FROM loadouts \
                    WHERE loadouts.id = $2";
            let query_no_user = "SELECT id, user_id, name, data, created_at, \
                    (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id) as like_count \
                    FROM loadouts \
                    WHERE loadouts.id = $1";

            conn.client.prepare(if with_user { query_with_user } else { query_no_user })
                .map_err(|e| l337::Error::External(e))
                .map(move |row| (conn, row, user.map(|u| u.id).unwrap_or(0)))
        })
        .from_err()
        .and_then(move |(mut conn, statement, user_id)| {
            let query = if with_user {
                conn.client.query(&statement, &[&user_id, &loadout_id])
            } else {
                conn.client.query(&statement, &[&loadout_id])
            };

            query
                .into_future()
                .map(|(r, _)| r)
                .map_err(|(e, _)| e)
                .from_err()
        })
        .and_then(move |row|
            Ok(row.map(|row| LoadoutSingle {
                id: row.get(0),
                user_id: row.get(1),
                name: row.get(2),
                data: row.get(3),
                created_at: row.get(4),
                like_count: row.get(5),
                has_liked: if with_user { row.get(6) } else { false },
            }))
        )
    }
}
