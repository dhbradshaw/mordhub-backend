use crate::{
    app::{self, PgConn, PgPool},
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
    pub has_liked: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadoutMultiple {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub data: String,
    pub created_at: NaiveDateTime,
    pub like_count: i64,
    pub has_liked: bool,
    pub main_image_url: String,
    pub user_steam_id: SteamId,
}

impl LoadoutMultiple {
    pub fn query(
        user: Option<User>,
        pool: &PgPool,
    ) -> impl Future<Item = Vec<Self>, Error = app::Error> {
        let with_user = user.is_some();

        // f3d5pzxulmlbpanpf5sc
        pool.connection()
            .from_err::<app::Error>()
            .and_then(move |mut conn| {
                let conn = &mut *conn;
                let query = if with_user {
                    conn.client.query(
                        &conn.queries.loadout_multiple_with_user,
                        &[&user.unwrap().id],
                    )
                } else {
                    conn.client
                        .query(&conn.queries.loadout_multiple_without_user, &[])
                };

                query.collect().from_err()
            })
            .and_then(move |rows| {
                Ok(rows
                    .into_iter()
                    .map(|row| LoadoutMultiple {
                        id: row.get(0),
                        user_id: row.get(1),
                        name: row.get(2),
                        data: row.get(3),
                        created_at: row.get(4),
                        like_count: row.get(5),
                        user_steam_id: row.get(6),
                        main_image_url: row.get(7),
                        has_liked: if with_user { row.get(8) } else { false },
                    })
                    .collect())
            })
    }
}

impl LoadoutSingle {
    pub fn query(
        loadout_id: i32,
        user: Option<User>,
        conn: &mut PgConn,
    ) -> impl Future<Item = Option<Self>, Error = app::Error> {
        let with_user = user.is_some();

        // f3d5pzxulmlbpanpf5sc
        let conn = &mut *conn;
        let query = if with_user {
            let user_id = user.map(|u| u.id).unwrap_or(0);
            conn.client.query(
                &conn.queries.loadout_single_with_user,
                &[&user_id, &loadout_id],
            )
        } else {
            conn.client
                .query(&conn.queries.loadout_single_without_user, &[&loadout_id])
        };

        query
            .into_future()
            .map(|(r, _)| r)
            .map_err(|(e, _)| e)
            .from_err()
            .and_then(move |row| {
                Ok(row.map(|row| LoadoutSingle {
                    id: row.get(0),
                    user_id: row.get(1),
                    name: row.get(2),
                    data: row.get(3),
                    created_at: row.get(4),
                    like_count: row.get(5),
                    has_liked: if with_user { row.get(6) } else { false },
                }))
            })
    }
}
