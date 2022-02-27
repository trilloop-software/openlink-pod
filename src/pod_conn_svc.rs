use super::pod_packet::*;
use super::pod_packet_payload::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::{mpsc::Receiver, mpsc::Sender, Mutex}, io::AsyncWriteExt, io::AsyncReadExt};

#[derive(Serialize, Deserialize)]
pub enum PodState {
    Unlocked,
    Locked,
    Moving,
    Braking
}

pub struct PodConnSvc {
    pub conn_list: Vec<TcpStream>,
    
    pub device_list: Arc<Mutex<Vec<super::device::Device>>>,
    pub pod_state: Arc<Mutex<PodState>>,

    pub rx_ctrl: Receiver<u8>,
    pub tx_ctrl: Sender<u8>,
    pub rx_emerg: Receiver<u8>,
    pub tx_emerg: Sender<u8>,
    pub rx_link: Receiver<u8>,
    pub tx_link: Sender<u8>,
    //pub rx_tele: Receiver<i32>,
    //pub tx_tele: Sender<i32>,
}

/// pod_conn_svc opens and manages tcp streams to all embedded devices
/// sends all embedded commands
/// receives responses from embedded devices
impl PodConnSvc {
    pub async fn run(mut self) {
        println!("pod_conn_svc: service running");

        loop {
            tokio::select! {
                //handle commands from ctrl_svc
                //ctrl_cmd = self.rx_ctrl.recv() => {
                //    self.send_cmd(0,ctrl_cmd.unwrap())
                //},

                _ = self.rx_emerg.recv() => {
                    // check pod_state
                    match *self.pod_state.lock().await {
                        PodState::Moving => {
                            // initiate braking
                            // send_cmd(emergengcy brake);
                            // braking command successful, return success to emerg_svc
                            if let Err(e) = self.tx_emerg.send(1).await {
                                eprintln!("pod->emerg failed: {}", e);
                            };
                        },
                        _ => {
                            // braking command unnecessary, return fail message to emerg_svc
                            if let Err(e) = self.tx_emerg.send(0).await {
                                eprintln!("pod->emerg failed: {}", e);
                            };
                        }
                    }

                }

                //handle commands from link_svc
                link_cmd = self.rx_link.recv() => {

                    //parse the command, and act based on it's command type
                    match link_cmd.unwrap() {
                        // lock cmd  
                        // start up all tcp connections
                        // if successful, set pod state to locked state and trigger discovery packet
                        // 
                        // if unsuccessful report unchanged state to user?
                        1 => {

                            match self.populate_conn_list().await {
                                Ok(()) => {

                                    //send the discovery packet command
                                    //to each device
                                    //for testing purposes, only send it to the first device
                                    let res = match self.send_cmd(0,1).await{
                                        Ok(()) => 1,
                                        Err(()) => 0,
                                    };
                                    if let Err(e) = self.tx_link.send(res).await {
                                        eprintln!("pod->link failed: {}", e);
                                    };
                                },
                                Err(()) => {

                                },
                            };
                            
                        },
                        _ => ()
                    }
                },
                /*tele_cmd = self.rx_tele.recv() => {
                    self.get_telemetry()
                }*/
            }
        }
    }

    fn get_telemetry(&mut self) {
        // iterate through conn_list TcpStreams
        // send commands to receive all telemetry
        // return to telemetry_svc
    }

    async fn populate_conn_list(&mut self) -> Result<(), ()> {
        if !self.conn_list.is_empty() {
            self.conn_list.clear()
        }

        // create TcpStream for each device in devicelist, and push to conn_list
        for dev in self.device_list.lock().await.clone() {
            let addr = format!("{}:{}", dev.ip_address, dev.port);
            match TcpStream::connect(addr).await {
                Ok(s) => self.conn_list.push(s),
                Err(_) => {println!("couldn't connect ")}
            }
        }

        // if devicelist.length == conn_list.length then all devices are connected and pod_state is locked
        if self.device_list.lock().await.len() != self.conn_list.len() {
            return Err(())
        }
        *self.pod_state.lock().await = PodState::Locked;

        Ok(())
    }

    async fn send_cmd(&mut self, index:usize, cmd: u8)-> Result<(), ()> {

        // send command to associated devices
        //  -returns array of available commands and array of device fields
        //  -figure out best way to store this and query it
        println!("sending cmd to device");

        //contruct the packet
        //build payload contents based on command type

        //let mut payload = PodPacketPayload::new();
        //for b in encode_payload(payload){
        //    println!("{}",b);
        //}

        let mut payload = PodPacketPayload::new();
        let packet = encode(PodPacket::new(cmd, encode_payload(payload)));

        //send packet to the device
        match self.conn_list[index].write_all(&packet).await{
            Ok(res) => println!("successfully sent command"),
            Err(e) => println!("failed to send command: {}", s!(e))
        };

        //read the packet that is returned by the device
        let mut buf = vec![0; 1024];
        match self.conn_list[index].read(&mut buf).await{
            Ok(size) => {

                println!("received response to command");

                //decode the response to the command
                let resp = decode(buf[0..size].to_vec());
                let payload = decode_payload(resp.payload);
                println!("decoded response to command");

                //process the response, based on the type of command that it is responding to
                //TODO: check that the cmd_type of the response matches up
                match cmd {
                    //response to an emergency command
                    255 =>{

                    },
                    //error response 
                    0 =>{

                    }
                    //response to a discovery command
                    1 =>{

                        //TODO store these in the Device struct
                        println!("Discovered Telemetry Fields:");
                        println!("{}",payload.field_names[0]);
                        println!("{}",payload.field_names[1]);
                        println!("{}",payload.field_names[2]);
                        println!("Discovered Commands:");
                        println!("{}",payload.commands[0]);
                        println!("{}",payload.commands[1]);
                        println!("{}",payload.commands[2]);
                    },
                    // commands 2-255 are not reserved for any particular command 
                    // (unlike 0 for emergency or 1 for discovery)
                    // so they need to be matched to the commands for device that sent the response packet
                    2..=255 =>{
                        //retrieve the list of commands for the device that sent the packet
                        //match the packet's cmd_type to the appropriate device-specific command
                    }
                }
                
            },
            Err(e) => println!("failed to send command: {}", s!(e))
        };

        Ok(())

    }
}

