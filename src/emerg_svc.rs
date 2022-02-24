use anyhow::{Result};
use tokio::{/*spawn, */sync::{/*broadcast, */mpsc::Receiver, mpsc::Sender/*, Mutex*/}};

//right now it looks like emergency service only receives commands from auth
//but I will still set it up so that it can receive commands from multiple services, just in case.

use super::packet::*;

pub struct EmergSvc{ //connections to auth, brake, telemetry
    pub rx_auth : Receiver<Packet>,
    pub tx_auth : Sender<Packet>,

    pub rx_pod_status : Receiver<Packet>,
    pub tx_pod_status : Sender<Packet>

    //pub rx_tele : Receiver<Packet>,
    //pub tx_tele : Sender<Packet>
}

impl EmergSvc
{
    pub async fn run(mut self) -> Result<()>
    {
        println!("emerg_svc running");

        loop //same concept as brake_svc from here out
        {
            tokio::select!{
                _val = self.rx_auth.recv() => {
                    println!("Auth->Emerg received");
                    let pkt = Packet { //packet to send to brake_svc
                        packet_id: s!["OPENLINK"],
                        version: 1,
                        cmd_type: 99, 
                        timestamp: std::time::SystemTime::now(),
                        payload: vec![1.to_string()]
                    };

                    match self.tx_pod_status.send(pkt).await{ //send to brake and get response, then send response back to auth
                        Ok(()) => { //send it back to auth
                            let res = self.rx_pod_status.recv().await.unwrap();
                            if let Err(e) = self.tx_auth.send(res).await {
                                eprintln!("Emerg->Auth failed: {}", e);
                            }
                        },
                        Err(e) => println!("Emerg->Brake failed, Error: {}", e)
                    }
                    
                }
            }
        }
    }
}