use anyhow::{Result};
use tokio::{/*spawn,*/ sync::{/*broadcast, */mpsc::Receiver, mpsc::Sender/*, Mutex*/}};

use super::{packet::*};

pub struct LaunchSvc {
    pub rx_auth: Receiver<Packet>,
    pub tx_auth: Sender<Packet>,
    pub rx_brake: Receiver<Packet>,
    pub tx_brake: Sender<Packet>
}

impl LaunchSvc {

    pub async fn run(mut self) -> Result<()> {
        println! ("launch_svc running!");


        loop {
            tokio::select!{ // mmight have to add brake svc
                _val = self.rx_auth.recv() => {
                    println!("auth -> launch received");
                    let response_ = Packet {
                        packet_id: s!["OPENLINK"],
                        version: 1,
                        cmd_type: 65,
                        timestamp: std::time::SystemTime::now(),
                        payload: vec!["Launch".to_string()]

                    };

                    if let Err(e) = self.tx_auth.send(response_).await{
                        eprintln!("Launch->Auth failed: {}", e);
                    }
                }

                _val = self.rx_brake.recv() => {
                    println!("brake -> launch received");
                    let response_ = Packet {
                        packet_id: s!["OPENLINK"],
                        version: 1,
                        cmd_type: 65,
                        timestamp: std::time::SystemTime::now(),
                        payload: vec!["Launch".to_string()]
                    };

                    if let Err(e) = self.tx_brake.send(response_).await{
                        eprintln!("Launch->Brake failed: {}", e);
                    }
                }
            }
        }
    }
}
