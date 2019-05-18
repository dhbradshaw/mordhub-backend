use crate::schema::loadouts;

#[derive(Debug, Clone, Insertable, Queryable, Serialize)]
#[table_name = "loadouts"]
pub struct Loadout {
    id: i32,
    user_id: i32,
    title: String,
    main_image_id: Option<i32>,
}
