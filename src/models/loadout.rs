use crate::models::{user::SteamId, User};
use crate::schema::loadouts;
use chrono::naive::NaiveDateTime;
use diesel::prelude::*;
use diesel::sql_types::*;

#[derive(Debug, Clone, Serialize, QueryableByName)]
pub struct LoadoutSingle {
    #[sql_type = "Integer"]
    pub id: i32,
    #[sql_type = "Integer"]
    pub user_id: i32,
    #[sql_type = "Varchar"]
    pub name: String,
    #[sql_type = "Varchar"]
    pub data: String,
    #[sql_type = "Timestamp"]
    pub created_at: NaiveDateTime,
    #[sql_type = "BigInt"]
    pub like_count: i64,
    #[sql_type = "Bool"]
    pub has_liked: bool, // Whether the current user has already liked this loadout
}

#[derive(Debug, Clone, QueryableByName, Serialize)]
pub struct LoadoutMultiple {
    #[sql_type = "Integer"]
    pub id: i32,
    #[sql_type = "Integer"]
    pub user_id: i32,
    #[sql_type = "Varchar"]
    pub name: String,
    #[sql_type = "Varchar"]
    pub data: String,
    #[sql_type = "Timestamp"]
    pub created_at: NaiveDateTime,
    #[sql_type = "BigInt"]
    pub like_count: i64,
    #[sql_type = "Bool"]
    pub has_liked: bool, // Whether the current user has already liked this loadout
    #[sql_type = "Varchar"]
    pub main_image_url: String,
    #[sql_type = "BigInt"]
    pub user_steam_id: SteamId,
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "loadouts"]
pub struct NewLoadout {
    pub user_id: i32,
    pub name: String,
    pub data: String,
}

impl LoadoutMultiple {
    pub fn query(
        user: Option<User>,
        conn: &PgConnection,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        if let Some(user) = user {
            diesel::sql_query(
                "SELECT loadouts.*, \
                    (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id) as like_count, \
                    EXISTS (SELECT user_id FROM likes WHERE user_id = $1) AS has_liked, \
                    (SELECT steam_id FROM users WHERE users.id = loadouts.user_id) as user_steam_id, \
                    (SELECT url FROM images WHERE images.loadout_id = loadouts.id AND images.position = 0) as main_image_url \
                FROM loadouts"
            )
                .bind::<Integer, _>(user.id)
                .get_results(conn)
        } else {
            diesel::sql_query(
                "SELECT loadouts.*, \
                    (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id) as like_count, \
                    (SELECT steam_id FROM users WHERE users.id = loadouts.user_id) as user_steam_id, \
                    (SELECT url FROM images WHERE images.loadout_id = loadouts.id AND images.position = 0) as main_image_url \
                FROM loadouts"
            )
                .get_results::<Self>(conn)
                .map(|res| res.into_iter().map(|mut r| { r.has_liked = false; r }).collect())
        }
    }
}

impl LoadoutSingle {
    pub fn query(
        id: i32,
        user: Option<User>,
        conn: &PgConnection,
    ) -> Result<Self, diesel::result::Error> {
        if let Some(user) = user {
            diesel::sql_query(
                "SELECT loadouts.*, \
                 (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id) as like_count, \
                 EXISTS (SELECT user_id FROM likes WHERE user_id = $1) AS has_liked \
                 FROM loadouts \
                 WHERE loadouts.id = $2",
            )
            .bind::<Integer, _>(user.id)
            .bind::<Integer, _>(id)
            .get_result::<Self>(conn)
        } else {
            diesel::sql_query(
                "SELECT loadouts.*, \
                 (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id) as like_count \
                 FROM loadouts \
                 WHERE loadouts.id = $1",
            )
            .bind::<Integer, _>(id)
            .get_result::<Self>(conn)
            .map(|mut r| {
                r.has_liked = false;
                r
            })
        }
    }
}
