use rusqlite::{params, Connection, Result};

pub fn cleanup(conn: &Connection) -> Result<()> {
    conn.execute("DROP TABLE IF EXISTS dbinfo", [])?;
    conn.execute("DROP TABLE IF EXISTS devices", [])?;
    conn.execute("DROP TABLE IF EXISTS telemetry", [])?;
    conn.execute("DROP TABLE IF EXISTS users", [])?;

    Ok(())
}

pub fn create(conn: &Connection) -> Result<()> {
    // create db_info table
    match conn.execute(
        "CREATE TABLE db_info (
                id      INTEGER PRIMARY KEY,
                version REAL
                )",
        [],
    ) {
        Ok(_) => println!("database_svc: db_info table created"),
        Err(_) => println!("database_svc: ERROR db_info table was not created")
    };

    // initialize db version
    match conn.execute(
        "INSERT INTO db_info (id, version) VALUES (?1, ?2)",
        params![0, super::DB_VER]
    ) {
        Ok(_) => println!("database_svc: db_info table initialized"),
        Err(_) => println!("database_svc: ERROR db_info table was not initialized")
    };

    // create devices table
    match conn.execute(
        "CREATE TABLE devices (
                id          STRING PRIMARY KEY,
                name        STRING,
                device_type BLOB,
                ip_address  STRING,
                port        INTEGER
                )",
        []
    ) {
        Ok(_) => println!("database_svc: devices table created"),
        Err(e) => eprintln!("database_svc: ERROR devices table was not created, {}", e)
    };

    // create telemetry table
    match conn.execute(
        "CREATE TABLE telemetry (
                time        INTEGER,
                data        BLOB
                )",
        []
    ) {
        Ok(_) => println!("database_svc: telemetry table created"),
        Err(e) => eprintln!("database_svc: ERROR telemetry table was not created, {}", e)
    };

    // create users table
    match conn.execute(
        "CREATE TABLE users (
                id      INTEGER PRIMARY KEY,
                name    STRING
                )",
        []
    ) {
        Ok(_) => println!("database_svc: users table created"),
        Err(e) => eprintln!("database_svc: ERROR users table was not created, {}", e)
    };

    Ok(())
}
