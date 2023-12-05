use rusqlite::Connection;

/// Get the current time in milliseconds
pub fn current_msec() -> f64 {
    let now = std::time::SystemTime::now();
    let since_the_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    since_the_epoch.as_millis() as f64
}

/// Open an SQLite database
pub fn open_database(path: &str) -> rusqlite::Result<Connection> {
    let db = Connection::open(path)?;
    write_schema(&db, include_str!("schema.sql"))?;
    Ok(db)
}

/// Write the database schema
pub fn write_schema(conn: &Connection, schema: &str) -> rusqlite::Result<()> {
    conn.execute_batch(schema)
}
