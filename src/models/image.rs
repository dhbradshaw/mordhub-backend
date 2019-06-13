use crate::app::{self, PgConn};
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
        conn: &mut PgConn,
    ) -> impl Future<Item = Vec<Self>, Error = app::Error> {
        let conn = &mut *conn;
        conn.client
            .query(&conn.queries.get_image_by_id, &[&loadout_id])
            .and_then(|row| {
                Ok(Image {
                    id: row.get(0),
                    url: row.get(1),
                    loadout_id: row.get(2),
                    position: row.get(3),
                    created_at: row.get(4),
                })
            })
            .from_err()
            .collect()
    }
}
