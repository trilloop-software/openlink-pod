use anyhow::{Result};
use tokio::sync::{mpsc::Receiver, mpsc::Sender, Mutex};
use serde_json;
use std::{sync::Arc, ops::Range};
/* POD STATE COMMANDS
64 - Get state
Returns PodState
68 - Set Destination
Sets launch_params
69 - Launch
Launches pod
99 - Brakes
Payload should be a bool to engage/disengage
*/

use shared::{launch::*, remote_conn_packet::*};
use crate::pod_packet::PodPacket;
use crate::pod_packet_payload::*;

use super::pod_conn_svc::PodState;
const DIST_RANGE: Range<f32> = 0.0..250.0;
const SPEED_RANGE: Range<f32> = 0.0..111.0;

pub struct CtrlSvc { 
    pub launch_params: LaunchParams,

    //State things
    pub pod_state: Arc<Mutex<PodState>>,

    //connections to other services
    pub rx_auth : Receiver<RemotePacket>,
    pub tx_auth : Sender<RemotePacket>,

    pub rx_pod : Receiver<PodPacket>,
    pub tx_pod: Sender<PodPacket>,

    pub tx_trip: Sender<LaunchParams>,
}

impl CtrlSvc {
    /// Main service task for controls service
    pub async fn run(mut self) -> Result<()> {
        println!("ctrl_svc: service running");

        while let Some(pkt) = self.rx_auth.recv().await {
            println!("Command type: {}", pkt.cmd_type);

            let resp = match pkt.cmd_type {
                //64 is the beginning of the command space for ctrl_svc
                64 => self.get_state().await.unwrap(),
                68 => self.set_destination(pkt.payload[0].clone()).await.unwrap(),
                69 => self.launch_pod().await.unwrap(),
                99 => self.engage_brakes().await.unwrap(),
                _ => RemotePacket::new(0, vec![s!("Invalid command")]),
                //127 is the end of the command space for ctrl_svc
            };

            if let Err(e) = self.tx_auth.send(resp).await {
                eprintln!("ctrl->auth failed: {}", e);
                break;
            }
        }

        println!("ctrl_svc: service down");

        Ok(())
    }

    /// Return the current state of the pod to the remote client
    async fn get_state(&mut self) -> Result<RemotePacket, ()> {
        if let Ok(pod_status_json) = serde_json::to_string(&*self.pod_state.lock().await) {
            Ok(RemotePacket::new(65, vec![pod_status_json]))
        } else {
            Ok(RemotePacket::new(0, vec![s!("Podstate unavailable")]))
        }
    }
    
    /// Launch the pod if in valid state
    async fn launch_pod(&mut self) -> Result<RemotePacket, ()> {

        let launch = match *self.pod_state.lock().await {
            PodState::Locked => true,
            _ => false
        };

        if launch {
            // send launch command to pod_conn_svc
            if let Err(e) = self.tx_pod.send(PodPacket::new(254,encode_payload(PodPacketPayload::new()))).await {
                eprintln!("ctrl->pod failed: {}", e);
            }

            //receive the ACK from pod_conn_svc
            self.rx_pod.recv().await;

            println!("ctrl: received ACK from pod_conn");

            // Once OK() is received, change state to PodState::Moving
            *self.pod_state.lock().await = PodState::Moving;

            if let Err(e) = self.tx_trip.send(self.launch_params.clone()).await {
                eprintln!("ctrl->trip failed: {}", e);
            }

            println!("Pod launched");
            // return the appropriate ACK packet wrapped in OK()
            return Ok(RemotePacket::new(69, vec![s!("Pod launched")]))
        } else {
            return Ok(RemotePacket::new(0, vec![s!("PodState not locked, cannot launch")]))
        }

    }

    /// Engage brakes if in valid state
    async fn engage_brakes(&mut self) -> Result<RemotePacket, ()> {
        let moving = match *self.pod_state.lock().await {
            PodState::Moving => true,
            _ => false
        };

        if moving {
            if let Err(e) = self.tx_pod.send(PodPacket::new(255, encode_payload(PodPacketPayload::new()))).await {
                eprintln!("ctrl->pod failed: {}", e);
            }

            self.rx_pod.recv().await;

            *self.pod_state.lock().await = PodState::Braking;
            println!("Pod braking");

            return Ok(RemotePacket::new(96, vec![s!("Pod brakes engaged")]))
        } else {
            return Ok(RemotePacket::new(0, vec![s!("PodState not moving, cannot brake")]))
        }
    }

    /// Set launch_params to be used by pod_conn_svc
    async fn set_destination(&mut self, req: String) -> Result<RemotePacket, ()> {
        if let Ok(params) = serde_json::from_str::<LaunchParams>(&req) {
            match params.distance {
                None => return Ok(RemotePacket::new(0, vec![s!("Invalid distance")])),
                Some(d) => {
                    if DIST_RANGE.contains(&d) {
                        match params.max_speed {
                            None => return Ok(RemotePacket::new(0, vec![s!("Invalid max speed")])),
                            Some(s) => {
                                if SPEED_RANGE.contains(&s) {
                                    self.launch_params = params;
                                    return Ok(RemotePacket::new(65, vec![s!("Launch parameters set")]))                
                                } else {
                                    return Ok(RemotePacket::new(0, vec![s!("Max speed out of valid range")]))
                                }
                            }
                        }
                    } else {
                        return Ok(RemotePacket::new(0, vec![s!("Distance out of valid range")]))
                    }
                }
            }
        } else {
            return Ok(RemotePacket::new(0, vec![s!("Launch parameters not set, malformed")]))
        }
    }
}