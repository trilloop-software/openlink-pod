use anyhow::{/*anyhow, */Result};
use serde_json;
use std::sync::Arc;
use tokio::sync::{mpsc::Receiver, mpsc::Sender, Mutex};

use super::{device::*, packet::*};

pub struct LinkSvc {
    pub device_list: Arc<Mutex<Vec<Device>>>,
    pub rx_auth: Receiver<Packet>,
    pub tx_auth: Sender<Packet>,
    pub rx_pod: Receiver<u8>,
    pub tx_pod: Sender<u8>,
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
                32 => self.get_device_list().await.unwrap(),
                33 => self.add_device(pkt.payload[0].clone()).await.unwrap(),
                34 => self.update_device(pkt.payload[0].clone()).await.unwrap(),
                35 => self.remove_device(pkt.payload[0].clone()).await.unwrap(),
                63 => self.lock_pod().await.unwrap(),
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

    /// Add new device to device list
    /// 
    /// Return success message to client
    async fn add_device(&mut self, req: String) -> Result<String, serde_json::Error> {
        println!("link_svc: add_device command received");
        let dev: Device = serde_json::from_str(&req)?;
        self.device_list.lock().await.push(dev);
        println!("link_svc: device added");

        Ok(s!["Device added"])
    }

    async fn get_device_list(&self) -> Result<String, serde_json::Error> {
        println!("link_svc: get_device_list command received");
        serde_json::to_string(&self.device_list.lock().await.clone())
    }

    /// Lock device_list to start TCP connections to embedded devices
    /// in pod_conn_svc
    async fn lock_pod(&self) -> Result<String, serde_json::Error> {
        println!("link_svc: lock_devices command received");

        //send lock command to pod_conn_svc
        if let Err(e) = self.tx_pod.send(1).await {
            println!("link->pod failed: {}", e);
        }

        // returning new device_list
        self.get_device_list().await
    }

    /// Find device received from client in device list and remove from vector
    /// 
    /// Return success message to client
    async fn remove_device(&mut self, req: String) -> Result<String, serde_json::Error> {
        println!("link_svc: remove_device command received");
        let dev: Device = serde_json::from_str(&req)?;
        let index = self.device_list.lock().await.iter().position(|d| d.id == dev.id).unwrap();
        self.device_list.lock().await.remove(index);
        println!("link_svc: device removed");

        Ok(s!["Device removed"])
    }

    /// Find device received from client in device list and update where id matches
    /// 
    /// Return success message to the client
    async fn update_device(&mut self, req: String) -> Result<String, serde_json::Error> {
        println!("link_svc: update_device command received");
        let dev: Device = serde_json::from_str(&req)?;
        let index = self.device_list.lock().await.iter().position(|d| d.id == dev.id).unwrap();
        self.device_list.lock().await[index] = dev;
        println!("link_svc: device updated");

        Ok(s!["Device updated"])
    }

    /// Temporary function to populate the device list
    async fn populate_temp_data(&mut self) {
        self.device_list.lock().await.push(Device { 
            id: s!("yhvlwn1"),
            name: s!("Battery 1"),
            device_type: DeviceType::Battery,
            ip_address: "127.0.0.1".parse().unwrap(),
            port: 0,
            connection_status: ConnectionStatus::Connected,
            device_status: DeviceStatus::Operational,
            fields: vec![ 
                DeviceField { field_name: s!("Temperature"), field_value: s!("") },
                DeviceField { field_name: s!("Power"), field_value: s!("") }
            ] ,
            commands: vec![] 
        });

        self.device_list.lock().await.push(Device { 
            id: s!("j5n4ook"),
            name: s!("Inverter 1"),
            device_type: DeviceType::Inverter,
            ip_address: "127.0.0.1".parse().unwrap(),
            port: 0,
            connection_status: ConnectionStatus::Connected,
            device_status: DeviceStatus::Unsafe,
            fields: vec![ 
                DeviceField { field_name: s!("Inverter Field 1"), field_value: s!("") },
                DeviceField { field_name: s!("Inverter Field 2"), field_value: s!("") }
            ] ,
            commands: vec![] 
        });

        self.device_list.lock().await.push(Device { 
            id: s!("573vxfk"),
            name: s!("Sensor 1"),
            device_type: DeviceType::Sensor,
            ip_address: "127.0.0.1".parse().unwrap(),
            port: 0,
            connection_status: ConnectionStatus::Disconnected,
            device_status: DeviceStatus::Unsafe,
            fields: vec![] ,
            commands: vec![]  
        });
    }
}
