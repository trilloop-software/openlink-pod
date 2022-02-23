use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Serialize, Deserialize, Clone)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub ip_address: Ipv4Addr,
    pub port: u16,
    pub connection_status: ConnectionStatus,
    pub device_status: DeviceStatus,
    pub fields: Vec<DeviceField>
}

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
pub struct DeviceField {
    pub field_name: String,
    pub field_value: String
}
