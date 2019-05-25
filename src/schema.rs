table! {
    images (id) {
        id -> Int4,
        url -> Varchar,
        loadout_id -> Int4,
        position -> Int4,
        created_at -> Timestamp,
    }
}

table! {
    likes (id) {
        id -> Int4,
        user_id -> Int4,
        loadout_id -> Int4,
    }
}

table! {
    loadouts (id) {
        id -> Int4,
        user_id -> Int4,
        name -> Varchar,
        data -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int4,
        steam_id -> Int8,
    }
}

joinable!(images -> loadouts (loadout_id));
joinable!(likes -> loadouts (loadout_id));
joinable!(likes -> users (user_id));
joinable!(loadouts -> users (user_id));

allow_tables_to_appear_in_same_query!(images, likes, loadouts, users,);
