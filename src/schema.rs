table! {
    images (id) {
        id -> Int4,
        url -> Varchar,
        uploader_id -> Int4,
        upload_date -> Nullable<Date>,
    }
}

table! {
    loadouts (id) {
        id -> Int4,
        user_id -> Int4,
        title -> Varchar,
        main_image_id -> Nullable<Int4>,
    }
}

table! {
    users (id) {
        id -> Int4,
        steam_id -> Int8,
    }
}

joinable!(images -> users (uploader_id));
joinable!(loadouts -> images (main_image_id));
joinable!(loadouts -> users (user_id));

allow_tables_to_appear_in_same_query!(images, loadouts, users,);
