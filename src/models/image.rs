use chrono::naive::NaiveDateTime;

#[derive(Debug, Serialize)]
pub struct Image {
    pub id: i32,
    pub url: String,
    pub loadout_id: i32,
    pub position: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct NewImage {
    pub url: String,
    pub loadout_id: i32,
    pub position: i32,
}
