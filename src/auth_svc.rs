use anyhow::{Result};
use tokio::sync::{mpsc::Receiver, mpsc::Sender};

use openlink_packets::{remote_conn_packet::*};

pub struct AuthSvc {
    pub rx_remote: Receiver<RemotePacket>,
    pub tx_remote: Sender<RemotePacket>,

    pub rx_link: Receiver<RemotePacket>,
    pub tx_link: Sender<RemotePacket>,
    
    pub rx_ctrl: Receiver<RemotePacket>,
    pub tx_ctrl: Sender<RemotePacket>
}

impl AuthSvc {
    /// Main service task for auth service
    /// Utilizing as command parser for now since all commands will require authentication in the future
    pub async fn run(mut self) -> Result<()> {
        println!("auth_svc running");

        while let Some(pkt) = self.rx_remote.recv().await {
            // send packet to associated service based on cmd_type field range
            println!("Packet of type {} received", pkt.cmd_type);
            let resp: RemotePacket = match pkt.cmd_type {
                0..=31 => {
                    // auth service command handling
                    pkt
                },
                32..=63 => {
                    // link service command handling
                    if let Err(e) = self.tx_link.send(pkt).await {
                        eprintln!("auth->link failed: {}", e);
                    }
                    self.rx_link.recv().await.unwrap()
                },
                64..=127 => {
                    // pod state service command handling
                    if let Err(e) = self.tx_ctrl.send(pkt).await {
                        eprintln!("auth->launch failed: {}", e);
                    }
                    self.rx_ctrl.recv().await.unwrap()
                },
                128..=159 => {
                    // telemetry service command handling
                    pkt
                },
                160..=195 => {
                    // database service command handling
                    pkt
                },
                196..=227 => {
                    // extra?
                    // 
                    pkt
                },
                228..=255 => {
                    // extra?
                    pkt
                },
            };

            // send the modified packet back to remote_conn_svc
            if let Err(e) = self.tx_remote.send(resp).await {
                eprintln!("auth->remote failed: {}", e)
            }
        }

        println!("auth_svc down");

        Ok(())
    }
}
