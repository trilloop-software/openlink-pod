use rusqlite::{Connection, params};
use super::super::user::*;

// cmd_type = 160
pub fn add_user(conn: &Connection, user: User) -> bool {
    match conn.execute(
        "INSERT INTO users (name, hash, ugroup) VALUES (?1), (?2), (?3)",
        params![user.name, user.hash, user.ugroup]) {
        Ok(_) => return true,
        Err(_) => return false
    }
}

// cmd_type = 161
pub fn get_user(conn: &Connection, name: String) -> User {
    match conn.query_row(
        "SELECT * FROM users WHERE name = (?1)",
        params![name],
        |row| {
            Ok(User::from_sql(
                row.get(0).unwrap(), 
                row.get(1).unwrap(), 
                row.get(2).unwrap()))}) {
                    Ok(user) => return user,
                    Err(_) => return User::new(s!(""), s!(""), 0)
                };
}

// cmd_type = 162
pub fn get_userlist(conn: &Connection) -> Vec<User> {
    let mut stmt = conn.prepare("SELECT name, hash, ugroup FROM users").unwrap();
    let mut rows = stmt.query([]).unwrap();
    let mut users = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        users.push(User::from_sql(
            row.get(0).unwrap(),
            row.get(1).unwrap(),
            row.get(2).unwrap()))
    }
    
    users
}

// cmd_type = 163
pub fn remove_user(conn: &Connection, name: String) -> bool {
    match conn.execute(
        "DELETE FROM users WHERE name = (?1)", 
        params![name]) {
        Ok(_) => return true,
        Err(_) => return false
    }
}

// cmd_type = 164
pub fn update_user_group(conn: &Connection, user: User) -> bool {
    match conn.execute(
        "UPDATE users SET ugroup=(?2) WHERE name=(?1)",
        params![user.name, user.ugroup]) {
        Ok(_) => return true,
        Err(_) => return false
    }
}
