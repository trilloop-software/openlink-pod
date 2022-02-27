use anyhow::{Result};
use tokio::sync::{mpsc::Receiver, mpsc::Sender, Mutex};
use serde_json;
use std::sync::Arc;
/* POD STATE COMMANDS
64 - Get state
Returns PodState
69 - Launch
Launches pod
99 - Brakes
Payload should be a bool to engage/disengage
*/

use super::packet::*;
use super::pod_conn_svc::PodState;

pub struct CtrlSvc { 

    //State things
    pub pod_state: Arc<Mutex<PodState>>,

    //connections to other services
    pub rx_auth : Receiver<Packet>,
    pub tx_auth : Sender<Packet>,

    pub rx_pod : Receiver<u8>,
    pub tx_pod: Sender<u8>
}

impl CtrlSvc {
    pub async fn run(mut self) -> Result<()> {
        println!("pod_state_svc: service running");

        loop {
            tokio::select!{
                pkt = self.rx_auth.recv() => {
                    //TODO: UNCOMMENT
                    //println!("auth -> pod_state received");

                    let mut response = Packet { //default packet
                        packet_id: s!["OPENLINK"],
                        version: 1,
                        cmd_type: 0,
                        timestamp: std::time::SystemTime::now(),
                        payload: vec!["Error".to_string()]
                    };

                    match pkt {
                        Some(packet) => response = self.command_handler(packet).await.unwrap(),
                        None => {
                            //TODO: UNCOMMENT
                            //println!("No packet here?!")
                        },
                    }

                    if let Err(e) = self.tx_auth.send(response).await{
                        //TODO: UNCOMMENT
                        //eprintln!("PodState->Auth failed: {}", e);
                    }
                }
            }
        }
    }

    async fn command_handler(&mut self, pkt: Packet) -> Result<Packet, serde_json::Error> {
        println!("Command type: {}", pkt.cmd_type);
        let res: Packet = match pkt.cmd_type {
            64 => self.get_state().await.unwrap(),
            69 => self.launch_pod().await.unwrap(),
            99 => self.engage_brakes().await.unwrap(),
            _ => Packet::new(0, vec![s!("Invalid command")]),
        };
        Ok(res)
    }

    async fn get_state(&mut self) -> Result<Packet, serde_json::Error> {
        let pod_status_json = serde_json::to_string(&*self.pod_state.lock().await).unwrap();
        Ok(Packet::new(65, vec![pod_status_json]))
    }
    
    async fn launch_pod(&mut self) -> Result<Packet, serde_json::Error> {
        match *self.pod_state.lock().await {
            PodState::Locked => {
                // send launch command to pod_conn_svc
                // wrap Ok() in await of recv channel from pod_conn_svc
                // change state to PodState::Moving in pod_conn_svc or here?
                Ok(Packet::new(69, vec![s!("Pod launched")]))
            },
            _ => return Ok(Packet::new(0, vec![s!("PodState not locked, cannot launch")])),
        }
    }

    async fn engage_brakes(&mut self) -> Result<Packet, serde_json::Error> {
        match *self.pod_state.lock().await {
            PodState::Moving => {
                // send braking command to pod_conn_svc
                // wrap Ok() in await of recv channel from pod_conn_svc
                // change state to PodState::Braking in pod_conn_svc or here?
                Ok(Packet::new(96, vec![s!("Pod brakes engaged")]))
            },
            _ => return Ok(Packet::new(0, vec![s!("PodState not moving, cannot brake")]))
        }
    }
}