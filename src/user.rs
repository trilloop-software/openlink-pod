use boringauth::pass::derive_password;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub hash: String,
    pub ugroup: u8,
}
// ugroup = permissions
// 0 -> no permissions
// 1 -> ???
// 2 -> ???
// 255 -> admin

impl User {
    pub fn new(name: String, pwd: String, ugroup: u8) -> Self {
        Self {
            name: name.to_lowercase(),
            hash: match derive_password(&pwd) {
                Ok(hash) => hash,
                Err(_) => s!(""),
            },
            ugroup,
        }
    }

    pub fn from_sql(name: String, hash: String, ugroup: u8) -> Self {
        Self { name, hash, ugroup }
    }
}
