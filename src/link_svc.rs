use anyhow::{anyhow, Result};
use tokio::sync::{mpsc::Receiver, mpsc::Sender};

pub struct LinkSvc {
    pub rx: Receiver<String>,
    pub tx: Sender<String>
}

impl LinkSvc {
    pub async fn run(mut self) -> Result<()> {
        println!("link_svc running");
        while let Some(_cmd) = self.rx.recv().await {
            if let Err(e) = self.tx.send(get_device_list()).await {
                eprintln!("What the hell just happened: {}", e);
            }
        }

        println!("link_svc done");

        Ok(())
    }
}

fn get_device_list() -> String {
    let data = r#"
        [
            { 
                "connection_status": 1,
                "device_status": 1,
                "device_type": "BATTERY",
                "fields": [
                    { "field_name": "Temperature", "field_value": "" },
                    { "field_name": "Power", "field_value": "" }
                ],
                "icon": "battery_full",
                "id": "yhvlwn1",
                "ip_address": "127.0.0.1",
                "name": "Battery 1",
                "port": 0
            },
            { 
                "connection_status": 1,
                "device_status": 0,
                "device_type": "INVERTER",
                "fields": [
                    { "field_name": "Inverter Field 1", "field_value": "" },
                    { "field_name": "Inverter Field 2", "field_value": "" }
                ],
                "icon": "bolt",
                "id": "j5n4ook",
                "ip_address": "127.0.0.1",
                "name": "Inverter 1",
                "port": 0
            },
            { 
                "connection_status": 0,
                "device_status": 0,
                "device_type": "SENSOR",
                "fields": [
                ],
                "icon": "speed",
                "id": "573vxfk",
                "ip_address": "127.0.0.1",
                "name": "Sensor 1",
                "port": 0
            }
        ]"#.to_string();

    data
}