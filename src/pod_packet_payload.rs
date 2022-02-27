use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize};

#[derive(Serialize, Deserialize)]

//template used in discovery packets
pub struct PodPacketPayload {

    pub field_names: Vec<String>,
    pub field_values: Vec<String>, //field data is parsed to a string for now. Since there are too many possible data types (u8,i32,etc)
    pub command_names: Vec<String>,
    pub command_values: Vec<u8>,
}

impl PodPacketPayload {
    pub fn new() -> Self {
        Self {
            field_names: Vec::new(),
            field_values: Vec::new(), 
            command_names: Vec::new(),
            command_values: Vec::new(),
        }
    }
    
}

pub fn decode_payload(pkt: Vec<u8>) -> PodPacketPayload {
    deserialize(&pkt[..]).unwrap()
}

pub fn encode_payload(pkt: PodPacketPayload) -> Vec<u8> {
    serialize(&pkt).unwrap()
}