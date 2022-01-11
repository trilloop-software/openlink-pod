use anyhow::{/*anyhow, */Result};
use serde_json;
use tokio::sync::{mpsc::Receiver, mpsc::Sender};

use super::{device::*, packet::*};

pub struct LinkSvc {
    pub device_list: Vec<Device>,
    pub rx: Receiver<Packet>,
    pub tx: Sender<Packet>
}

impl LinkSvc {
    // main service task for link service
    pub async fn run(mut self) -> Result<()> {
        println!("link_svc running");
        self.populate_temp_data();

        while let Some(mut pkt) = self.rx.recv().await {
            // process response based on cmd_type variable
            let res = match pkt.cmd_type {
                32 => self.get_device_list().unwrap(),
                33 => self.add_device(pkt.payload[0].clone()).unwrap(),
                34 => self.update_device(pkt.payload[0].clone()).unwrap(),
                35 => self.remove_device(pkt.payload[0].clone()).unwrap(),
                _ => s!["Command not implemented"]
            };

            // clear packet payload and add the response to payload vector
            pkt.payload.clear();
            pkt.payload.push(res);

            // send modified packet to auth_svc
            if let Err(e) = self.tx.send(pkt).await {
                eprintln!("link->auth failed: {}", e);
            }
        }

        println!("link_svc down");

        Ok(())
    }

    // add new device to device list
    // return success message to client
    fn add_device(&mut self, req: String) -> Result<String, serde_json::Error> {
        println!("add_device command received");
        let dev: Device = serde_json::from_str(&req)?;
        self.device_list.push(dev);
        println!("success: device added");

        Ok(s!["Success: Device added."])
    }

    fn get_device_list(&self) -> Result<String, serde_json::Error> {
        println!("get_device_list command received");
        serde_json::to_string(&self.device_list)
    }

    // find device received from client in device list and remove from vector
    // return success message to client
    fn remove_device(&mut self, req: String) -> Result<String, serde_json::Error> {
        println!("remove_device command received");
        let dev: Device = serde_json::from_str(&req)?;
        let index = self.device_list.iter().position(|d| d.id == dev.id).unwrap();
        self.device_list.remove(index);
        println!("success: device removed");

        Ok(s!["Success: Device removed."])
    }

    // find device received from client in device list and update where id matches
    // return success message to the client
    fn update_device(&mut self, req: String) -> Result<String, serde_json::Error> {
        println!("update_device command received");
        let dev: Device = serde_json::from_str(&req)?;
        let index = self.device_list.iter().position(|d| d.id == dev.id).unwrap();
        self.device_list[index] = dev;
        println!("success: device updated");

        Ok(s!["Success: Device updated."])
    }

    // temporary function to populate the device list
    fn populate_temp_data(&mut self) {
        self.device_list.push(Device { 
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
            ] 
        });

        self.device_list.push(Device { 
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
            ] 
        });

        self.device_list.push(Device { 
            id: s!("573vxfk"),
            name: s!("Sensor 1"),
            device_type: DeviceType::Sensor,
            ip_address: "127.0.0.1".parse().unwrap(),
            port: 0,
            connection_status: ConnectionStatus::Disconnected,
            device_status: DeviceStatus::Unsafe,
            fields: vec![] 
        });
    }
}
