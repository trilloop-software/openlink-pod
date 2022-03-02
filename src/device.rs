use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::fmt;

#[derive(Serialize, Deserialize, Clone)]
pub enum DeviceType {
    Battery,
    Inverter,
    Sensor
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ConnectionStatus {
    Disconnected,
    Connected
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DeviceStatus {
    Unsafe,
    Operational
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub ip_address: Ipv4Addr,
    pub port: u16,
    pub connection_status: ConnectionStatus,
    pub device_status: DeviceStatus,
    pub fields: Vec<DeviceField>,
    pub commands: Vec<DeviceCommand>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceField {
    pub field_name: String,
    pub field_value: String
}

impl DeviceField{
    pub fn new(name:String ) -> Self{
        Self{
            field_name:name,
            field_value:s![""],
        }
        
    }

    pub fn update_value(mut self, value:String){
        self.field_value = value;
    }

}

impl fmt::Display for DeviceField{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field_name, self.field_value)
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceCommand {
    pub cmd_name: String,
    pub cmd_value: u8
}

impl DeviceCommand{
    pub fn new(name:String, value:u8 ) -> Self{
        Self{
            cmd_name:name,
            cmd_value:value,
        }
        
    }

    pub fn update_value(mut self, value:u8){
        self.cmd_value = value;
    }

}

impl fmt::Display for DeviceCommand{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name: {}, Code: {}", self.cmd_name, self.cmd_value)
    }
}
