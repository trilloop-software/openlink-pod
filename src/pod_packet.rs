use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize};

#[derive(Serialize, Deserialize)]
pub struct PodPacket {
    pub packet_id: String,
    pub version: u8,
    pub cmd_type: u8,
    // no timestamp because embedded devices may not have system time
    pub payload: Vec<String>
}

impl PodPacket {
    pub fn new(cmd_type: u8, payload: Vec<String>) -> Self {
        Self {
            packet_id: "OPENLINK".to_string(),
            version: 1,
            cmd_type: cmd_type,
            payload: payload
        }
    }
}

pub fn decode(pkt: Vec<u8>) -> PodPacket {
    deserialize(&pkt[..]).unwrap()
}

pub fn encode(pkt: PodPacket) -> Vec<u8> {
    serialize(&pkt).unwrap()
}
