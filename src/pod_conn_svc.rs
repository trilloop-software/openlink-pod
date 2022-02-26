use super::pod_packet::*;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::{mpsc::Receiver, mpsc::Sender, Mutex}, io::AsyncWriteExt, io::AsyncReadExt};
use futures_util::stream::StreamExt;
use quinn::{Endpoint, ServerConfig};
use std::{error::Error};
use tracing::{error, info};

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
                //ctrl_cmd = self.rx_ctrl.recv() => {
                //    self.send_cmd(0,ctrl_cmd.unwrap())
                //},

                //handle commands from link_svc
                link_cmd = self.rx_link.recv() => {

                    match link_cmd.unwrap() {
                        // lock cmd  
                        // start up all tcp connections
                        // if successful, set pod state to locked state and trigger discovery packet
                        // 
                        // if unsuccessful report unchanged state to user?
                        1 => {

                            
                            match self.populate_conn_list().await {
                                Ok(()) => {
                                    let res = match self.send_cmd(0,1).await{
                                        Ok(()) => 1,
                                        Err(()) => 0,
                                    };
                                    self.tx_link.send(res).await;
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
        // discovery packet should be cmd #1
        //  -returns array of available commands and array of device fields
        //  -figure out best way to store this and query it
        println!("sending cmd to device");

        //contruct the packet
        //let packet = encode(Packet::new(0, vec![s!("Test")]));

        //for b in packet{
        //    println!("{}",b);
        //}

        let packet = encode(PodPacket::new(0, vec![s!("Test")]));
        //send it to the device
        let res = match self.conn_list[index].write_all(&packet).await{
            Ok(res) => println!("success"),
            Err(e) => println!("failed to send command: {}", s!(e))
        };

        //read the packet that is returned by the device
        let mut buf = vec![0; 1024];
        let res = match self.conn_list[index].read(&mut buf).await{
            Ok(size) => {

                println!("received response to command");

                //try to decode the response to the command
                let pckt = decode(buf[0..size].to_vec());

                println!("decoded response to command");
                
            },
            Err(e) => println!("failed to send command: {}", s!(e))
        };

        Ok(())

    }
}

