use anyhow::{anyhow, Result};
use serde_json;
use tokio::sync::{mpsc::Receiver, mpsc::Sender};

use super::{device::*};

pub struct LinkSvc {
    pub device_list: Vec<Device>,
    pub rx: Receiver<String>,
    pub tx: Sender<String>
}

impl LinkSvc {
    // main service task for link service
    pub async fn run(mut self) -> Result<()> {
        println!("link_svc running");
        self.populate_temp_data();

        while let Some(_cmd) = self.rx.recv().await {
            if let Err(e) = self.tx.send(self.get_device_list().unwrap()).await {
                eprintln!("What the hell just happened: {}", e);
            }
        }

        println!("link_svc done");

        Ok(())
    }

    fn get_device_list(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.device_list)
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
