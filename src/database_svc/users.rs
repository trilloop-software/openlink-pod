use rusqlite::{Connection, params};

use shared::user::*;
use super::super::{RemotePacket, user::*};

pub fn handler(conn: &Connection, mut pkt: RemotePacket) -> RemotePacket {
    match pkt.cmd_type {
        160 => {
            if let Ok(user) = serde_json::from_str::<UserRaw>(&pkt.payload[0]) {
                let user = User::new(user.name, user.pwd, user.ugroup);

                if add_user(conn, user) {
                    pkt.payload[0] = s!("User added");
                } else {
                    pkt = pkt.error(s!("User add failed"));
                }
            } else {
                pkt = pkt.error(s!("Malformed user information"));
            }

            pkt
        },
        161 => {
            if let Ok(user) = serde_json::from_str::<User>(&pkt.payload[0]) {
                pkt.payload[0] = serde_json::to_string(&get_user(&conn, user.name)).unwrap();
            } else {
                pkt = pkt.error(s!("Malformed user information"));
            }

            pkt
        },
        162 => {
            let userlist: Vec<UserSecure> = get_user_list(&conn);
            pkt.payload[0] = serde_json::to_string(&userlist).unwrap();
            pkt
        },
        163 => {
            if pkt.payload[0] == "admin" {
                pkt = pkt.error(s!("Cannot remove admin account"));
            } else {
                if remove_user(&conn, pkt.payload[0].clone()) {
                    pkt.payload[0] = s!("User removed");
                } else {
                    pkt = pkt.error(s!("User remove failed"));
                }
            }

            pkt
        },
        164 => {
            if let Ok(user) = serde_json::from_str::<UserSecure>(&pkt.payload[0]) {
                if user.name == "admin" {
                    pkt = pkt.error(s!("Cannot change admin account permissions"))
                } else {
                    if update_user_group(&conn, user) {
                        pkt.payload[0] = s!("User group updated");
                    } else {
                        pkt = pkt.error(s!("User group update failed"));
                    }
                }
            } else {
                pkt = pkt.error(s!("Malformed user information"))
            }

            pkt
        },
        165 => {
            if let Ok(user) = serde_json::from_str::<UserRaw>(&pkt.payload[0]) {
                let user = User::new(user.name, user.pwd, user.ugroup);
                
                if update_user_password(&conn, user) {
                    pkt.payload[0] = s!("User password updated")
                } else {
                    pkt = pkt.error(s!("User password update failed"))
                }
            } else {
                pkt = pkt.error(s!("Malformed user information"))
            }

            pkt
        },
        _ => {
            pkt
        }
    }
}

// cmd_type = 160
pub fn add_user(conn: &Connection, user: User) -> bool {
    match conn.execute(
        "INSERT INTO users (name, hash, ugroup) VALUES (?1, ?2, ?3)",
        params![user.name, user.hash, user.ugroup]) {
        Ok(_) => return true,
        Err(e) => return false
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
pub fn get_user_list(conn: &Connection) -> Vec<UserSecure> {
    let mut stmt = conn.prepare("SELECT name, ugroup FROM users").unwrap();
    let mut rows = stmt.query([]).unwrap();
    let mut users = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        users.push(UserSecure::from_sql(
            row.get(0).unwrap(),
            row.get(1).unwrap()))
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
pub fn update_user_group(conn: &Connection, user: UserSecure) -> bool {
    match conn.execute(
        "UPDATE users SET ugroup=(?2) WHERE name=(?1)",
        params![user.name, user.ugroup]) {
        Ok(_) => return true,
        Err(_) => return false
    }
}

// cmd_type = 165
pub fn update_user_password(conn: &Connection, user: User) -> bool {
    match conn.execute(
        "UPDATE users SET hash=(?2) WHERE name=(?1)",
        params![user.name, user.hash]) {
            Ok(_) => return true,
            Err(_) => return false
        }
}
