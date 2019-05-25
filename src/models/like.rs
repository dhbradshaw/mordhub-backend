use crate::schema::likes;

#[derive(Debug, Queryable, Insertable)]
#[table_name = "likes"]
pub struct Like {
    pub id: i32,
    pub user_id: i32,
    pub loadout_id: i32,
}

#[derive(Debug, Insertable)]
#[table_name = "likes"]
pub struct NewLike {
    pub user_id: i32,
    pub loadout_id: i32,
}
