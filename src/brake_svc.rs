use anyhow::{Result};
use tokio::{/*spawn, */sync::{/*broadcast, */mpsc::Receiver, mpsc::Sender/*, Mutex*/}};

use super::packet::*;

pub struct BrakeSvc{ //connections to other services
    pub rx_auth : Receiver<Packet>,
    pub tx_auth : Sender<Packet>,

    pub rx_launch : Receiver<Packet>,
    pub tx_launch : Sender<Packet>,

    pub rx_emerg : Receiver<Packet>,
    pub tx_emerg: Sender<Packet>
}

impl BrakeSvc
{
    pub async fn run(mut self) -> Result<()> {
        println!("brake_svc running");

        loop
        {
            tokio::select!{
                _val = self.rx_auth.recv() => { //there should be a better way to do this, via a function call or something, but every time I try to do it I run into borrowing/mutability/lifetime issues
                    //if anyone else has an idea on how to solve that, give it a try.
                    println!("Auth->Brake received"); 
                    let res = Packet {
                        packet_id: s!["OPENLINK"],
                        version: 1,
                        cmd_type: 96,
                        timestamp: std::time::SystemTime::now(),
                        payload: vec!["Brakes engaged".to_string()]
                    };
                    if let Err(e) = self.tx_auth.send(res).await {
                        eprintln!("Brake->Auth failed: {}", e);
                    }
                }
                _val = self.rx_launch.recv() => {
                    println!("Launch->Brake received");
                    let res = Packet {
                        packet_id: s!["OPENLINK"],
                        version: 1,
                        cmd_type: 96,
                        timestamp: std::time::SystemTime::now(),
                        payload: vec!["Brakes engaged".to_string()]
                    };
                    if let Err(e) = self.tx_launch.send(res).await {
                        eprintln!("Brake->Auth failed: {}", e);
                    }
                }
                _val = self.rx_emerg.recv() => {
                    println!("Emerg-> Brake received");
                    let res = Packet {
                        packet_id: s!["OPENLINK"],
                        version: 1,
                        cmd_type: 96,
                        timestamp: std::time::SystemTime::now(),
                        payload: vec!["Brakes engaged".to_string()]
                    };
                    if let Err(e) = self.tx_emerg.send(res).await {
                        eprintln!("Brake->Auth failed: {}", e);
                    }
                }
            }
        }
    }

}