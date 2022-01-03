pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub icon: DeviceTypeIcon,
    pub ip_address: Ipv4Addr,
    pub port: u16,
    pub connection_status: ConnectionStatus,
    pub device_status: DeviceStatus,
    pub fields: Vec<DeviceFields>
}

pub enum DeviceType {
    Battery = "BATTERY",
    Inverter = "INVERTER",
    Sensor = "SENSOR"
}

pub enum DeviceTypeIcon {
    Battery = "battery_full",
    Inverter = "bolt",
    Sensor = "speed"
}

pub enum ConnectionStatus {
    Disconnected,
    Connected
}

pub enum DeviceStatus {
    Unsafe,
    Operational
}

pub struct DeviceFields {
    pub field_name: String,
    pub field_value: String
}
