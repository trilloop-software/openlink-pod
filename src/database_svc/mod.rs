use rusqlite::{Connection, Result};

mod schema;

const DB_VER: f32 = 0.1;
const ADMIN_PASS: &str = "password";

pub struct DatabaseSvc {
    //pub rx_auth: Receiver<>,
    //pub tx_auth: Sender<>,
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
            /*tokio::select! {
                /*_ = self.rx_auth.recv() => {

                }

                _ = self.rx_link.recv() => {

                }

                _ = self.rx_tele.recv() => {

                }*/
            }*/
        }
    }
}
