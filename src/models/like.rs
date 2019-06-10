#[derive(Debug)]
pub struct Like {
    pub id: i32,
    pub user_id: i32,
    pub loadout_id: i32,
}

#[derive(Debug)]
pub struct NewLike {
    pub user_id: i32,
    pub loadout_id: i32,
}
