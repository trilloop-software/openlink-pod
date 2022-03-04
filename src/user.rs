use boringauth::pass::derive_password;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub hash: String,
    pub ugroup: u8
}
// ugroup = permissions
// 0 -> no permissions
// 1 -> ???
// 2 -> ???
// 255 -> admin

impl User {
    pub fn new(name: String, pwd: String, ugroup: u8) -> Self {
        Self {
            name,
            hash: match derive_password(&pwd) {
                Ok(hash) => hash,
                Err(_) => s!("")
            },
            ugroup
        }
    }

    pub fn change_pwd(mut self, pwd: String) {
        self.hash = derive_password(&pwd).unwrap();
    }

    pub fn change_ugroup(mut self, ugroup: u8) {
        self.ugroup = ugroup;
    }

    pub fn from_sql(name: String, hash: String, ugroup: u8) -> Self {
        Self {
            name,
            hash,
            ugroup
        }
    }
}
