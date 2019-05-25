use crate::schema::loadouts;
use chrono::naive::NaiveDateTime;

#[derive(Debug, Clone, Insertable, Queryable, Serialize)]
#[table_name = "loadouts"]
pub struct Loadout {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub data: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "loadouts"]
pub struct NewLoadout {
    pub user_id: i32,
    pub name: String,
    pub data: String,
}
