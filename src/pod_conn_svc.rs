use crate::pod_packet::{decode, encode, PodPacket};
use crate::pod_packet_payload::{decode_payload, encode_payload, PodPacketPayload};
use shared::device::{Device, DeviceCommand, DeviceField};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{
    io::AsyncReadExt,
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{mpsc::Receiver, mpsc::Sender, Mutex},
};

#[derive(Serialize, Deserialize)]
pub enum PodState {
    Unlocked,
    Locked,
    Moving,
    Braking,
}

pub struct PodConnSvc {
    pub conn_list: Vec<TcpStream>,

    pub device_list: Arc<Mutex<Vec<Device>>>,
    pub pod_state: Arc<Mutex<PodState>>,

    pub rx_ctrl: Receiver<PodPacket>,
    pub tx_ctrl: Sender<PodPacket>,
    pub rx_emerg: Receiver<u8>,
    pub tx_emerg: Sender<u8>,
    pub rx_link: Receiver<PodPacket>,
    pub tx_link: Sender<PodPacket>,
    //pub rx_tele: Receiver<i32>,
    //pub tx_tele: Sender<i32>,
    pub rx_trip: Receiver<u8>,
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

                //handle commands from ctrl_svc
                packet = self.rx_ctrl.recv() =>{

                    let mut pkt = packet.unwrap().clone();
                    let link_cmd = pkt.cmd_type;
                    let payload = decode_payload(pkt.payload);

                    //parse the command, and act based on it's command type
                    match link_cmd{
                        //cmd to engage brakes
                        255=>{
                            self.engage_brakes().await;

                            //send ACK back to ctrl_svc
                            self.tx_ctrl.send(PodPacket::new(254,encode_payload(PodPacketPayload::new()))).await;
                        }
                        //cmd to launch pod
                        254=>{
                            self.launch().await;
                        }
                        //cmd to activate device specific command
                        _=>{

                        }
                    }

                }

                //handle commands from link_svc
                packet = self.rx_link.recv() => {

                    let mut pkt = packet.unwrap().clone();
                    let link_cmd = pkt.cmd_type;
                    let payload = decode_payload(pkt.payload);

                    //parse the command, and act based on it's command type
                    match link_cmd{

                        // lock cmd
                        // start up all tcp connections
                        // if successful, set pod state to locked state and trigger discovery packet
                        //
                        // if unsuccessful report unchanged state to user?
                        1 => {

                            let mut unlocked =false;

                            //check if the pod is already locked before following through with the lock command
                            match *self.pod_state.lock().await {
                                PodState::Unlocked => {
                                    unlocked =true;
                                },
                                _ => {
                                    // locking command unnecessary, return fail message
                                        eprintln!("Pod already locked");
                                    //TODO: return an error message to link_svc

                                }
                            }

                            if unlocked {

                                //open TCP connections to all devices
                                match self.populate_conn_list().await {
                                    Ok(()) => {

                                        //send the discovery packet command
                                        //to each device

                                        let num = self.device_list.lock().await.len();

                                        for index in 0..num{
                                            let res = match self.send_cmd(index,1, PodPacketPayload::new()).await{
                                                Ok(()) => 1,
                                                Err(()) => 0,
                                            };
                                            if let Err(e) = self.tx_link.send(PodPacket::new(1,Vec::<u8>::new())).await {
                                                eprintln!("pod->link failed: {}", e);
                                            };
                                        }

                                        //once successful, send a response to link_svc
                                        pkt.payload = vec![0];
                                        self.tx_link.send(pkt).await;

                                    },
                                    Err(()) => {

                                        //if unsuccessful, send a response to link_svc
                                        pkt.payload = vec![0];
                                        pkt.cmd_type = 0;
                                        self.tx_link.send(pkt).await;

                                    },
                                };
                            }


                        },
                        // unlock cmd
                        // end all tcp connections
                        // if successful, set pod state to unlocked state
                        2 =>{
                            let mut unlocked =true;

                            //check if the pod is already locked before following through with the unlock command
                            match *self.pod_state.lock().await {
                                PodState::Locked => {
                                    unlocked = false;
                                },
                                PodState::Unlocked => {
                                    // locking command unnecessary, return fail message
                                    eprintln!("Pod already unlocked");

                                }
                                _ => {
                                    // unlocking command denied when pod is Moving or Braking
                                    //return fail message
                                    eprintln!("Pod may only be Unlocked when in Locked state");

                                }
                            }

                            //if pod is in Locked state
                            //follow through with setting it to Unlocked state
                            if !unlocked {
                                *self.pod_state.lock().await = PodState::Unlocked;

                                //close TCP connections to all devices
                                match self.clear_conn_list().await{
                                    Ok(()) =>{
                                        println!("device connections closed successfully");
                                        //once successful, send a response to link_svc
                                        pkt.payload = vec![0];
                                        self.tx_link.send(pkt).await;
                                    }
                                    Err(())=>{
                                        println!("error: could not close device connections");
                                        //if unsuccessful, send a response to link_svc
                                        pkt.payload = vec![0];
                                        pkt.cmd_type = 0;
                                        self.tx_link.send(pkt).await;
                                    }
                                };

                            }
                            else{
                                //if unsuccessful, send a response to link_svc
                                pkt.payload = vec![0];
                                pkt.cmd_type = 0;
                                self.tx_link.send(pkt).await;
                            }

                        }
                        //send cmd to device
                        3=>{
                            let index = self.device_list.lock().await.iter().position(|d| d.id == payload.target_id).unwrap();

                            let mut unlocked =true;

                            //check if the pod is already locked before sending the command
                            match *self.pod_state.lock().await {
                                PodState::Locked => {
                                    unlocked = false;
                                },
                                PodState::Unlocked => {
                                    // return fail message
                                    eprintln!("Please lock the pod first");

                                }
                                _ => {

                                    //return fail message
                                    eprintln!("Failed to send command to device");


                                }
                            }

                            //if pod is in Locked state
                            //follow through with sending the command to the target device
                            if !unlocked {

                                //the command code specified in the packet from link_svc
                                //is the command code that the device will recognize
                                if let Err(()) = self.send_cmd(index, payload.target_cmd_code, PodPacketPayload::new()).await {
                                    println!("pod_conn_svc: send_cmd failed");
                                }
                                //once successful, send a response to link_svc
                                pkt.payload = vec![0];
                                self.tx_link.send(pkt).await;

                            }
                            else{
                                //if unsuccessful, send a response to link_svc
                                pkt.payload = vec![0];
                                pkt.cmd_type = 0;
                                self.tx_link.send(pkt).await;
                            }

                        }
                        _ => ()
                    }
                },
                /*tele_cmd = self.rx_tele.recv() => {
                    self.get_telemetry()
                }*/
                _ = self.rx_trip.recv() => {
                    self.engage_brakes().await;
                }
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
                Err(_) => {
                    println!("couldn't connect ")
                }
            }
        }

        // if devicelist.length == conn_list.length then all devices are connected and pod_state is locked
        if self.device_list.lock().await.len() != self.conn_list.len() {
            return Err(());
        }
        *self.pod_state.lock().await = PodState::Locked;

        Ok(())
    }

    async fn clear_conn_list(&mut self) -> Result<(), ()> {
        let size = self.device_list.lock().await.len();

        for index in 0..size {
            if let Err(()) = self.send_cmd(index, 2, PodPacketPayload::new()).await {
                println!("pod_conn_svc: send_cmd failed");
            }
        }

        Ok(())
    }

    async fn engage_brakes(&mut self) {
        //send the braking command
        //to each device
        let num = self.device_list.lock().await.len();

        for index in 0..num {
            let res = match self.send_cmd(index, 255, PodPacketPayload::new()).await {
                Ok(()) => 1,
                Err(()) => 0,
            };
            if let Err(e) = self
                .tx_ctrl
                .send(PodPacket::new(255, Vec::<u8>::new()))
                .await
            {
                eprintln!("pod->ctrl failed: {}", e);
            };
        }
    }

    async fn launch(&mut self) {
        //send the launch command
        //to each device
        let num = self.device_list.lock().await.len();

        for index in 0..num {
            let res = match self.send_cmd(index, 254, PodPacketPayload::new()).await {
                Ok(()) => 1,
                Err(()) => 0,
            };
            if let Err(e) = self
                .tx_ctrl
                .send(PodPacket::new(255, Vec::<u8>::new()))
                .await
            {
                eprintln!("pod->ctrl failed: {}", e);
            };
        }
    }

    async fn send_cmd(
        &mut self,
        index: usize,
        cmd: u8,
        payload: PodPacketPayload,
    ) -> Result<(), ()> {
        //println!("sending cmd to device");

        //contruct the packet
        let packet = encode(PodPacket::new(cmd, encode_payload(payload)));

        //send packet to the device
        match self.conn_list[index].write_all(&packet).await {
            Ok(()) => println!("successfully sent command"),
            Err(e) => println!("failed to send command: {}", s!(e)),
        };

        //determine if response packet is expected
        if cmd == 2 {
        } else {
            //read the packet that is returned by the device
            let mut buf = vec![0; 1024];
            match self.conn_list[index].read(&mut buf).await {
                Ok(size) => {
                    //println!("received response to command");

                    //decode the response to the command
                    let resp = decode(buf[0..size].to_vec());
                    let payload = decode_payload(resp.payload);
                    //println!("decoded response to command");

                    //process the response, based on the type of command that it is responding to
                    //TODO: check that the cmd_type of the response matches up
                    match cmd {
                        //response to an emergency/braking command
                        255 => {
                            println!("pod_conn: Braking Sequence successful");
                        }
                        254 => {
                            println!("pod_conn: Launching Sequence successful");
                            //send ACK back to ctrl_svc
                            if let Err(e) = self
                                .tx_ctrl
                                .send(PodPacket::new(254, encode_payload(PodPacketPayload::new())))
                                .await
                            {
                                eprintln!("pod->ctrl failed: {}", e);
                            }
                        }
                        //error response
                        0 => {}
                        //response to a discovery command
                        1 => {
                            //extract the list of new field names
                            let mut field_list = Vec::<DeviceField>::new();
                            for field in payload.field_names {
                                field_list.push(DeviceField::new(field));
                            }

                            //extract the list of new command anmes and their corresponding codes
                            let mut cmd_list = Vec::<DeviceCommand>::new();
                            for index in 0..payload.command_codes.len() {
                                cmd_list.push(DeviceCommand::new(
                                    payload.command_names[index].clone(),
                                    payload.command_codes[index],
                                ));
                            }

                            // clone the target device from the shared device list
                            let mut new_device = self.device_list.lock().await[index].clone();

                            // make changes to an updated clone of the original Device instance
                            new_device.fields = field_list;
                            new_device.commands = cmd_list;

                            // DEBUGGING PURPOSES
                            println!("Device index: {}", index);
                            // print the new fields/commands
                            println!("Discovered Telemetry Fields:");
                            for field in new_device.fields.clone() {
                                println!("{}", field);
                            }

                            println!("Discovered Commands:");
                            for cmd in new_device.commands.clone() {
                                println!("{}", cmd);
                            }

                            println!("--------------------");

                            // overwrite the original device in the list
                            // with the updated clone
                            self.device_list.lock().await[index] = new_device;

                            // success message
                            println!("Discovered Fields and Commands saved");
                        }
                        //command #2 is reserved for disconnect commands
                        //should not receive a response
                        2 => {
                            println!("Error: received response to disconnect command");
                        }
                        //response to a launch sequence command
                        3 => {
                            println!("Launching Sequence successful");
                        }
                        // commands 4-254 are not reserved for any particular command
                        // (unlike 255 for emergency or 1 for discovery)
                        4..=254 => {
                            //retrieve the list of commands for the device that sent the packet
                            //match the packet's cmd_type to the appropriate device-specific command
                        }
                    }
                }
                Err(e) => println!("failed to send command: {}", s!(e)),
            };
        }

        Ok(())
    }
}
