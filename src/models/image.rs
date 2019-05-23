//use chrono::naive::NaiveDate;
use crate::schema::images;

#[derive(Debug, Insertable)]
#[table_name = "images"]
pub struct NewImage {
    pub url: String,
    pub uploader_id: i32,
}
