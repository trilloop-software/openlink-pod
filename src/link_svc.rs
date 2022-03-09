use anyhow::Result;
use serde_json;
use std::sync::Arc;
use tokio::sync::{mpsc::Receiver, mpsc::Sender, Mutex};

use shared::{remote_conn_packet::*, device::*};
use super::{pod_packet::*, pod_packet_payload::*, pod_conn_svc::PodState};

pub struct LinkSvc {
    pub device_list: Arc<Mutex<Vec<Device>>>,
    pub pod_state: Arc<Mutex<PodState>>,

    pub rx_auth: Receiver<RemotePacket>,
    pub tx_auth: Sender<RemotePacket>,
    pub rx_pod: Receiver<PodPacket>,
    pub tx_pod: Sender<PodPacket>,
}

/// link_svc adds/removes/modifies devices to/from the pod
/// 
impl LinkSvc {
    /// Main service task for link service
    pub async fn run(mut self) -> Result<()> {
        println!("link_svc: service running");
        //self.populate_temp_data().await;

        while let Some(mut pkt) = self.rx_auth.recv().await {
            // process response based on cmd_type variable
            let res = match pkt.cmd_type {
                //32 is the beginning of the command space for link_svc
                32 => self.get_device_list().await.unwrap(),
                33 => self.add_device(pkt.payload[0].clone()).await.unwrap(),
                34 => self.update_device(pkt.payload[0].clone()).await.unwrap(),
                35 => self.remove_device(pkt.payload[0].clone()).await.unwrap(),
                36 => self.send_device_cmd(pkt.target_cmd_code, pkt.payload[0].clone()).await.unwrap(),
                62 => self.unlock_pod().await.unwrap(),
                63 => self.lock_pod().await.unwrap(),
                //63 is the end of the command space for link_svc
                _ => s!["Command not implemented"]
            };

            // clear packet payload and add the response to payload vector
            pkt.payload.clear();
            pkt.payload.push(res);

            // send modified packet to auth_svc
            if let Err(e) = self.tx_auth.send(pkt).await {
                eprintln!("link->auth failed: {}", e);
                break;
            }
        }

        println!("link_svc: service down");

        Ok(())
    }

    async fn pod_is_unlocked(&mut self) -> bool {
        match *self.pod_state.lock().await{

            //return true if pod is in Unlocked state
            PodState::Unlocked =>{
                println!("Checking Pod State: Unlocked");
                true
            }
            //otherwise, return false
            _ => {
                println!("Checking Pod State: NOT Unlocked");
                false
            }
        }
    }

    /// Add new device to device list
    /// 
    /// Return success message to client
    async fn add_device(&mut self, req: String) -> Result<String, serde_json::Error> {

        //only follow through with the command if pod is in Unlocked state
        if self.pod_is_unlocked().await {
            println!("link_svc: add_device command received");
            let dev: Device = serde_json::from_str(&req)?;
            self.device_list.lock().await.push(dev);
            println!("link_svc: device added");
    
            Ok(s!["Device added"])
        }
        //otherwise, return a failure message
        else{
            Ok(s!["Pod must be unlocked first"])
        }


    }

    async fn get_device_list(&self) -> Result<String, serde_json::Error> {
        println!("link_svc: get_device_list command received");
        serde_json::to_string(&self.device_list.lock().await.clone())
    }

    /// Lock device_list to start TCP connections to embedded devices in pod_conn_svc
    /// Once locked, devices cannot be edited in "Configure" page until the pod is unlocked
    async fn lock_pod(&mut self) -> Result<String, serde_json::Error> {
        println!("link_svc: lock_devices command received");

        //send lock command to pod_conn_svc
        if let Err(e) = self.tx_pod.send(PodPacket::new(1,encode_payload(PodPacketPayload::new()))).await {
            //if unsuccessful
            //return error message
            println!("link->pod failed: {}", e);
        }

        // if lock command was successful
        match self.rx_pod.recv().await.unwrap().cmd_type{
            0 =>{
                println!("Lock failed");
                Ok(s!["lock unsuccessful"])
            }
            _=>{
                // return new device_list
                self.get_device_list().await;
                //return success message
                Ok(s!["lock successful"])
            }

        }


    }

    // Unlock device_list to stop TCP connections to embedded devices in pod_conn_svc
    // Once unlocked, devices can be re-configured in the "Configure page" until pod is locked again
    async fn unlock_pod(&mut self)-> Result<String, serde_json::Error>{
        println!("link_svc: unlock_devices command received");

        //send unlock command to pod_conn_svc
        if let Err(e) = self.tx_pod.send(PodPacket::new(2,encode_payload(PodPacketPayload::new()))).await {
            //if unsuccessful
            //return error message
            println!("link->pod failed: {}", e);
        }

        // if lock command was successful
        match self.rx_pod.recv().await.unwrap().cmd_type{
            0 =>{
                println!("Unlock failed");
                Ok(s!["unlock unsuccessful"])
            }
            _=>{
                //return success message
                Ok(s!["unlock successful"])
            }

        }
    }

    /// Find device received from client in device list and remove from vector
    /// 
    /// Return success message to client
    async fn remove_device(&mut self, req: String) -> Result<String, serde_json::Error> {

        //only follow through with the command if pod is in Unlocked state
        if self.pod_is_unlocked().await {
            println!("link_svc: remove_device command received");
            let dev: Device = serde_json::from_str(&req)?;
            let index = self.device_list.lock().await.iter().position(|d| d.id == dev.id).unwrap();
            self.device_list.lock().await.remove(index);
            println!("link_svc: device removed");
    
            Ok(s!["Device removed"])
        }
        //otherwise, return a failure message
        else{
            Ok(s!["Pod must be unlocked first"])
        }

    }

    /// Find device received from client in device list and update where id matches
    /// 
    /// Return success message to the client
    async fn update_device(&mut self, req: String) -> Result<String, serde_json::Error> {

        //only follow through with the command if pod is in Unlocked state
        if self.pod_is_unlocked().await {
            println!("link_svc: update_device command received");
            let dev: Device = serde_json::from_str(&req)?;
            let index = self.device_list.lock().await.iter().position(|d| d.id == dev.id).unwrap();
            self.device_list.lock().await[index] = dev;
            println!("link_svc: device updated");
    
            Ok(s!["Device updated"])
        }
        //otherwise, return a failure message
        else{
            Ok(s!["Pod must be unlocked first"])
        }

    }

    /// Get device and command code from client
    /// Send the command to the corresponding device in device list
    async fn send_device_cmd(&mut self, cmd_code: u8, req: String) -> Result<String, serde_json::Error> {
        println!("link_svc: send_device_cmd command received");

        //recontruct the Device instance from the payload
        let dev: Device = serde_json::from_str(&req)?;

        //construct a payload that specifies the target device and target cmd code
        let mut payload = PodPacketPayload::new();
        payload.target_id = dev.id;
        payload.target_cmd_code = cmd_code;

        //tell pod_conn_svc to send the command to the appropriate device
        if let Err(e) = self.tx_pod.send(PodPacket::new(3,encode_payload(payload))).await {
            //if unsuccessful
            //return error message
            println!("link->pod failed: {}", e);
        }

        //return success message
        Ok(s!["Cmd sent to device"])
    }

}
