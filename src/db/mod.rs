mod manager;
mod queries;

pub type ConnectionManager = manager::PostgresConnectionManager<tokio_postgres::NoTls>;
pub type Connection = manager::Connection;
