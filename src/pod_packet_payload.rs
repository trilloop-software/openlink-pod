use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize};

#[derive(Serialize, Deserialize)]

//template used in discovery packets
pub struct PodPacketPayload {

    pub field_names: Vec<String>,
    pub telemetry_data: Vec<u8>, //assume all data is of type u8 for now (maybe make this Vec<Vec<u8>> later?)
    pub command_names: Vec<String>,
    pub commands: Vec<u8>,
}

impl PodPacketPayload {
    pub fn new() -> Self {
        Self {
            field_names: Vec::new(),
            telemetry_data: Vec::new(), //assume all data is of type u8 for now (maybe make this Vec<byte array> later?)
            command_names: Vec::new(),
            commands: Vec::new(),
        }
    }
    
}

pub fn decode_payload(pkt: Vec<u8>) -> PodPacketPayload {
    deserialize(&pkt[..]).unwrap()
}

pub fn encode_payload(pkt: PodPacketPayload) -> Vec<u8> {
    serialize(&pkt).unwrap()
}