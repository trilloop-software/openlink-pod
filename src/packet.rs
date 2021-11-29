use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use std::time::SystemTime;

#[derive(Serialize, Deserialize)]
pub struct Packet {
    pub version: u8,
    pub source: SocketAddr,
    pub destination: SocketAddr,
    pub cmd_type: u8,
    pub timestamp: SystemTime,
    pub payload: Vec<String>
}

pub fn decode(pkt: Vec<u8>) -> Packet {
    deserialize(&pkt[..]).unwrap()
}

pub fn encode(pkt: Packet) -> Vec<u8> {
    serialize(&pkt).unwrap()
}
