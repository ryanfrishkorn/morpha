use rusqlite::Connection;

/// return a database connection
pub fn setup() -> Result<Connection, rusqlite::Error> {
    Connection::open_in_memory()
}
