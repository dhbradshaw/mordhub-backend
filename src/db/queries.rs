use futures::Future;
use tokio_postgres::{types::Type, Client};

macro_rules! typed_queries {
    ($($name:ident => $query:expr, $sql_types:expr;)*) => {
        #[derive(Builder)]
        pub struct Queries {
            $(pub $name: tokio_postgres::Statement,)*
        }

        impl Queries {
            pub fn prepare(
                mut client: Client,
            ) -> impl Future<Item = (Client, Self), Error = tokio_postgres::Error> {
                use std::sync::{Mutex, Arc};
                let builder = Arc::new(Mutex::new(QueriesBuilder::default()));

                let mut futures: Vec<Box<dyn Future<Item = (), Error = tokio_postgres::Error> + Send>> = Vec::new();

                $( futures.push(Box::new({
                    let builder = builder.clone();
                    client
                        .prepare_typed($query, &$sql_types)
                        .map(move |statement| {
                            let mut b = builder.lock().unwrap();
                            b.$name(statement);
                        })
                })); )*

                futures::future::join_all(futures).and_then(move |_| {
                    let built = builder.lock().unwrap().build().unwrap();
                    Ok((client, built))
                })
            }
        }
    }
}

// https://docs.rs/tokio-postgres/0.4.0-rc.2/tokio_postgres/types/struct.Type.html
typed_queries! {
    get_image_by_id => "SELECT id, url, loadout_id, position, created_at FROM images WHERE loadout_id = $1 ORDER BY position ASC", [Type::INT4];

    loadout_single_with_user =>
        "SELECT id, user_id, name, data, created_at, \
        (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id) as like_count, \
        EXISTS (SELECT user_id FROM likes WHERE user_id = $1) AS has_liked \
        FROM loadouts \
        WHERE loadouts.id = $2",
        [Type::INT4, Type::INT4];

    loadout_single_without_user =>
        "SELECT id, user_id, name, data, created_at, \
        (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id) as like_count \
        FROM loadouts \
        WHERE loadouts.id = $1",
        [Type::INT4];

    loadout_multiple_with_user =>
        "SELECT id, user_id, name, data, created_at, \
        (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id) as like_count, \
        (SELECT steam_id FROM users WHERE users.id = loadouts.user_id) as user_steam_id, \
        (SELECT url FROM images WHERE images.loadout_id = loadouts.id AND images.position = 0) as main_image_url, \
        EXISTS (SELECT user_id FROM likes WHERE user_id = $1) AS has_liked FROM loadouts",
        [Type::INT4];

    loadout_multiple_without_user =>
        "SELECT id, user_id, name, data, created_at, \
        (SELECT COUNT(*) FROM likes WHERE likes.loadout_id = loadouts.id) as like_count, \
        (SELECT steam_id FROM users WHERE users.id = loadouts.user_id) as user_steam_id, \
        (SELECT url FROM images WHERE images.loadout_id = loadouts.id AND images.position = 0) as main_image_url FROM loadouts",
        [];

    get_user_by_id => "SELECT id, steam_id FROM users WHERE steam_id = $1", [Type::INT4];

    post_login_insert_user => "INSERT INTO users (steam_id) VALUES ($1) ON CONFLICT DO NOTHING", [Type::INT8];

    create_loadout =>
        "INSERT INTO loadouts (user_id, name, data, created_at) VALUES ($1, $2, $3, DEFAULT) RETURNING id",
        [Type::INT4, Type::VARCHAR, Type::VARCHAR, Type::TIMESTAMP];

    create_image =>
        "INSERT INTO images (url, loadout_id, position) VALUES ($1, $2, $3)",
        [Type::VARCHAR, Type::INT4, Type::INT4];
}
