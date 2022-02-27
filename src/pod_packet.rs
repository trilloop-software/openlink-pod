use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize};

#[derive(Serialize, Deserialize)]
pub struct PodPacket {
    pub packet_id: String,
    pub version: u8,
    pub cmd_type: u8,
    // no timestamp because embedded devices may not have system time

    //the payload takes the form of another struct: PodPacketPayload
    //however, it is encoded as a Vec<u8>, and decoded seperately once the PodPacket is decoded
    pub payload: Vec<u8> 
}

impl PodPacket {
    pub fn new(cmd_type: u8, payload: Vec<u8>) -> Self {
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
