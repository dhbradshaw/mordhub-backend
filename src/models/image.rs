use crate::app::{self, PgPool};
use chrono::naive::NaiveDateTime;
use futures::{stream::Stream, Future};

#[derive(Debug, Serialize)]
pub struct Image {
    pub id: i32,
    pub url: String,
    pub loadout_id: i32,
    pub position: i32,
    pub created_at: NaiveDateTime,
}

impl Image {
    pub fn query(
        loadout_id: i32,
        pool: &PgPool,
    ) -> impl Future<Item = Vec<Self>, Error = app::Error> {
        pool.connection()
            .and_then(|mut conn| {
                conn.client.prepare(
                    "SELECT id, url, loadout_id, position, created_at FROM images WHERE loadout_id = $1 ORDER BY position ASC"
                )
                    .map_err(|e| l337::Error::External(e))
                    .map(|statement| (conn, statement))
            })
            .from_err()
            .and_then(move |(mut conn, statement)|
                conn.client.query(&statement, &[&loadout_id])
                    .and_then(|row| Ok(Image {
                        id: row.get(0),
                        url: row.get(1),
                        loadout_id: row.get(2),
                        position: row.get(3),
                        created_at: row.get(4),
                    }))
                    .from_err()
                    .collect()
            )
    }
}
