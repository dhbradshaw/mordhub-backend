use crate::models::User;
use crate::schema::loadouts;
use chrono::naive::NaiveDateTime;
use diesel::prelude::*;
use diesel::sql_types::*;

// TODO: There's FAR too much repetition of field types, names etc.

#[derive(Debug, Clone, Serialize)]
pub struct Loadout {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub data: String,
    pub created_at: NaiveDateTime,
    pub like_count: u64,
    pub has_liked: bool, // Whether the current user has already liked this loadout
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "loadouts"]
pub struct NewLoadout {
    pub user_id: i32,
    pub name: String,
    pub data: String,
}

#[derive(Debug, Clone, Queryable, Serialize)]
struct LoadoutWithLikes {
    id: i32,
    user_id: i32,
    name: String,
    data: String,
    created_at: NaiveDateTime,
    like_count: Option<i64>,
}

#[derive(Debug, Clone, Queryable, Serialize)]
struct LoadoutWithUserLikes {
    id: i32,
    user_id: i32,
    name: String,
    data: String,
    created_at: NaiveDateTime,
    like_count: Option<i64>,
    has_liked: bool,
}

impl Loadout {
    pub fn query_single(
        loadout_id: i32,
        user: Option<User>,
        conn: &PgConnection,
    ) -> Result<Self, diesel::result::Error> {
        let like_count = {
            use crate::schema::likes::dsl::*;
            likes
                .filter(loadout_id.eq(crate::schema::loadouts::id))
                .select(diesel::dsl::count_star())
                .single_value()
        };

        if let Some(user) = user {
            let has_liked = {
                use crate::schema::likes::dsl::*;
                likes
                    .filter(user_id.eq(user.id))
                    .filter(loadout_id.eq(crate::schema::loadouts::id))
            };

            use crate::schema::loadouts::dsl::*;
            loadouts
                .find(loadout_id as i32)
                .select((
                    id,
                    user_id,
                    name,
                    data,
                    created_at,
                    like_count,
                    diesel::dsl::exists(has_liked),
                ))
                .first::<LoadoutWithUserLikes>(conn)
                .map(Loadout::from)
        } else {
            use crate::schema::loadouts::dsl::*;
            loadouts
                .find(loadout_id as i32)
                .select((id, user_id, name, data, created_at, like_count))
                .first::<LoadoutWithLikes>(conn)
                .map(Loadout::from)
        }
    }

    pub fn query_multiple(conn: &PgConnection) -> Result<Vec<Self>, diesel::result::Error> {
        let like_count = {
            use crate::schema::likes::dsl::*;
            likes
                .filter(loadout_id.eq(crate::schema::loadouts::id))
                .select(diesel::dsl::count_star())
                .single_value()
        };

        use crate::schema::loadouts::dsl::*;
        loadouts
            // TODO: Filter results (pagination, gold / level required etc)
            .select((id, user_id, name, data, created_at, like_count))
            .limit(10)
            .load::<LoadoutWithLikes>(conn) // Sets has_liked to false
            .map(|res| res.into_iter().map(|ldt| Loadout::from(ldt)).collect())
    }
}

impl From<LoadoutWithUserLikes> for Loadout {
    fn from(l: LoadoutWithUserLikes) -> Self {
        Self {
            id: l.id,
            user_id: l.user_id,
            name: l.name,
            data: l.data,
            created_at: l.created_at,
            like_count: l.like_count.map(|i| i as u64).unwrap_or(0),
            has_liked: l.has_liked,
        }
    }
}

impl From<LoadoutWithLikes> for Loadout {
    fn from(l: LoadoutWithLikes) -> Self {
        Self {
            id: l.id,
            user_id: l.user_id,
            name: l.name,
            data: l.data,
            created_at: l.created_at,
            like_count: l.like_count.map(|i| i as u64).unwrap_or(0),
            has_liked: false,
        }
    }
}

#[derive(Debug, Clone, QueryableByName)]
pub struct LoadoutBetter {
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
}

impl LoadoutBetter {
    pub fn query_multiple(conn: &PgConnection) -> Result<Vec<Self>, diesel::result::Error> {
        dbg!(diesel::sql_query(
            "SELECT \
                loadouts.*, \
                (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id) as like_count, \
                EXISTS (SELECT user_id FROM likes WHERE user_id = 1) AS has_liked, \
                (SELECT url FROM images WHERE images.loadout_id = loadouts.id AND images.position = 0) as main_image_url \
            FROM loadouts")
            .load(conn))
    }
}
