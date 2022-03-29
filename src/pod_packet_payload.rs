use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]

//template used in discovery packets
pub struct PodPacketPayload {
    pub target_id: String,
    pub target_cmd_code: u8,
    pub field_names: Vec<String>,
    pub telemetry_data: Vec<u8>, //assume all data is of type u8 for now (maybe make this Vec<Vec<u8>> later?)
    pub command_names: Vec<String>,
    pub command_codes: Vec<u8>,
}

impl PodPacketPayload {
    pub fn new() -> Self {
        Self {
            target_id: s![""],
            target_cmd_code: 0,
            field_names: Vec::new(),
            telemetry_data: Vec::new(), //assume all data is of type u8 for now (maybe make this Vec<byte array> later?)
            command_names: Vec::new(),
            command_codes: Vec::new(),
        }
    }
}

pub fn decode_payload(pkt: Vec<u8>) -> PodPacketPayload {
    deserialize(&pkt[..]).unwrap()
}

pub fn encode_payload(pkt: PodPacketPayload) -> Vec<u8> {
    serialize(&pkt).unwrap()
}
