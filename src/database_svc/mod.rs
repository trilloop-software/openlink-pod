use rusqlite::{Connection, Result};
use tokio::sync::{mpsc::Receiver, mpsc::Sender};

use super::user::User;

pub mod devices;
mod schema;
pub mod telemetry;
pub mod users;

const DB_VER: f32 = 0.1;
const ADMIN_PASS: &str = "password";

pub struct DatabaseSvc {
    pub rx_auth: Receiver<super::RemotePacket>,
    pub tx_auth: Sender<super::RemotePacket>,
    //pub rx_link: Receiver<>,
    //pub tx_link: Sender<>,
    //pub rx_tele: Receiver<>,
    //pub tx_tele: Receiver<>,
}

impl DatabaseSvc {
    pub async fn run(mut self) -> Result<()> {
        println!("database_svc: service running");

        // create/open database
        let conn = Connection::open("openlink.db")?;

        match conn.query_row(
            "SELECT version FROM db_info WHERE id == 0",
            [],
            |row| row.get::<usize, f32>(0)) {
            // database exists, check version and remake if neccessary
            Ok(db_ver) => {
                if db_ver != DB_VER {
                    schema::cleanup(&conn)?;
                    schema::create(&conn)?;
                }
            },
            // database does not exist, create tables
            Err(_) => schema::create(&conn)?
        }

        loop {
            tokio::select! {
                Some(pkt) = self.rx_auth.recv() => {
                    let res = users::handler(&conn, pkt);
                    
                    if let Err(e) = self.tx_auth.send(res).await {
                        eprintln!("data->auth failed: {}", e);
                    }
                }

                /*_ = self.rx_link.recv() => {

                }

                _ = self.rx_tele.recv() => {

                }*/
            }
        }
    }
}
