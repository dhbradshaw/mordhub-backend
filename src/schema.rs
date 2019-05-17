table! {
    images (id) {
        id -> Int4,
        url -> Varchar,
        uploaded_by -> Int4,
        upload_date -> Date,
    }
}

table! {
    loadouts (id) {
        id -> Int4,
        user_id -> Int4,
        title -> Varchar,
        main_image_id -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        steam_id -> Int8,
    }
}

joinable!(images -> users (uploaded_by));
joinable!(loadouts -> images (main_image_id));
joinable!(loadouts -> users (user_id));

allow_tables_to_appear_in_same_query!(images, loadouts, users,);
