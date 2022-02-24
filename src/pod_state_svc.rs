use anyhow::{Result};
use tokio::{/*spawn, */sync::{/*broadcast, */mpsc::Receiver, mpsc::Sender/*, Mutex*/}};
use serde_json;
use serde::{ser::{Serialize, Serializer, SerializeStruct}};
/* POD STATE COMMANDS
64 - Get state
Returns vector of values?
69 - Launch
Launches pod
99 - Brakes
Payload should be a bool to engage/disengage
*/

use super::packet::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PodStatus
{
    pub brakes_engaged : bool,
    pub is_moving : bool,
    pub is_locked : bool,
}

pub struct PodStateSvc{ 

    //State things
    pub pod_status : PodStatus,

    //connections to other services
    pub rx_auth : Receiver<Packet>,
    pub tx_auth : Sender<Packet>,

    pub rx_emerg : Receiver<Packet>,
    pub tx_emerg: Sender<Packet>,

    //pub rx_pod_conn : Receiver<Packet>,
    //pub tx_pod_conn: Sender<Packet>
}

impl PodStateSvc
{
    pub async fn run(mut self) -> Result<()> {
        println!("pod_state_svc running");
        //some default values

        loop {
            tokio::select!{
                pkt = self.rx_auth.recv() => {
                    println!("auth -> pod_state received");

                    let mut response = Packet { //default packet
                        packet_id: s!["OPENLINK"],
                        version: 1,
                        cmd_type: 0,
                        timestamp: std::time::SystemTime::now(),
                        payload: vec!["Error".to_string()]
                    };

                    match pkt
                    {
                        Some(packet) => response = self.command_handler(packet).unwrap(),
                        None => println!("No packet here?!"),
                    }

                    if let Err(e) = self.tx_auth.send(response).await{
                        eprintln!("PodState->Auth failed: {}", e);
                    }
                }

                pkt = self.rx_emerg.recv() => {
                    println!("emerg -> pod_state received");

                    let mut response = Packet { //default packet
                        packet_id: s!["OPENLINK"],
                        version: 1,
                        cmd_type: 0,
                        timestamp: std::time::SystemTime::now(),
                        payload: vec!["Error".to_string()]
                    };

                    match pkt
                    {
                        Some(packet) => response = self.command_handler(packet).unwrap(),
                        None => println!("No packet here?!"),
                    }
                    if let Err(e) = self.tx_emerg.send(response).await{
                        eprintln!("PodState->Emerg failed: {}", e);
                    }
                }

                /*pkt = self.rx_pod_conn.recv() => {
                    println!("pod_conn -> pod_state received");

                    match pkt
                    {
                        Some(packet) => res = self.command_handler(packet).unwrap(),
                        None => println!("No packet here?!"),
                    }

                    if let Err(e) = self.tx_auth.send(res).await{
                        eprintln!("PodState->PodConn failed: {}", e);
                    }
                }*/
            }
        }
    }

    fn command_handler(&mut self, pkt: Packet) -> Result<Packet, serde_json::Error>
    {
        println!("Command type: {}", pkt.cmd_type);
        let res: Packet = match pkt.cmd_type {
            64 => self.get_state().unwrap(),
            69 => self.launch_pod().unwrap(),
            99 => self.brakes(pkt.payload[0].clone()).unwrap(),
            _ => Packet { //default packet
                packet_id: s!["OPENLINK"],
                version: 1,
                cmd_type: 1,
                timestamp: std::time::SystemTime::now(),
                payload: vec![s!("Invalid command")]
            },
        };
        Ok(res)
    }

    fn get_state(&mut self) -> Result<Packet, serde_json::Error>
    {
        let pod_status_json = serde_json::to_string(&self.pod_status).unwrap();
        let res = Packet { //default packet
            packet_id: s!["OPENLINK"],
            version: 1,
            cmd_type: 65,
            timestamp: std::time::SystemTime::now(),
            payload: vec![pod_status_json]
        };
        Ok(res)
    }
    
    fn launch_pod(&mut self) -> Result<Packet, serde_json::Error>
    {
        if !self.pod_status.is_locked
        {
            return Ok(Packet{
            packet_id: s!["OPENLINK"],
            version: 1,
            cmd_type: 69,
            timestamp: std::time::SystemTime::now(),
            payload: vec!["Pod not launched, devices not yet locked".to_string()]
            })
        }
        self.pod_status.brakes_engaged = false;
        self.pod_status.is_moving = true;

        let res = Packet { //default packet
            packet_id: s!["OPENLINK"],
            version: 1,
            cmd_type: 69,
            timestamp: std::time::SystemTime::now(),
            payload: vec!["Pod launched".to_string()]
        };

        Ok(res)
    }

    fn brakes(&mut self, cmd: String) -> Result<Packet, serde_json::Error>
    {
        let status : bool;
        if (cmd == "1")
        {
            status = true;
        }
        else
        {
            status = false;
        }
        println!("Status set to {}", status);
        self.pod_status.brakes_engaged = status;

        let res = Packet { //default packet
            packet_id: s!["OPENLINK"],
            version: 1,
            cmd_type: 96,
            timestamp: std::time::SystemTime::now(),
            payload: if status { vec!["Brakes engaged".to_string()] } else {vec!["Brakes disengaged".to_string()]}
        };

        Ok(res)
    }
}