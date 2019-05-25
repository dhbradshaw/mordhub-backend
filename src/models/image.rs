use crate::schema::images;
use chrono::naive::NaiveDateTime;

#[derive(Debug, Queryable, Insertable, Serialize)]
#[table_name = "images"]
pub struct Image {
    pub id: i32,
    pub url: String,
    pub loadout_id: i32,
    pub position: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "images"]
pub struct NewImage {
    pub url: String,
    pub loadout_id: i32,
    pub position: i32,
}
