use anyhow::Result;
use std::sync::Arc;
use tokio::{
    sync::{mpsc::Receiver, mpsc::Sender, Mutex},
    time::{sleep, Duration},
};

use super::pod_conn_svc::PodState;
use shared::launch::*;

pub struct TripSvc {
    pub pod_state: Arc<Mutex<PodState>>,

    pub rx_ctrl: Receiver<LaunchParams>,
    pub tx_pod: Sender<u8>,
}

impl TripSvc {
    pub async fn run(mut self) -> Result<()> {
        println!("trip_svc: service running");
        // wait for signal from ctrl_svc

        // calculate trip
        // set timer for half for moving state
        // send braking command to pod_conn when over
        // set timer for half for braking state
        // change state to locked
        while let Some(params) = self.rx_ctrl.recv().await {
            let time = (params.distance.unwrap() / (params.max_speed.unwrap() / 3.6)) / 2.0;

            sleep(Duration::from_secs_f32(time)).await;
            *self.pod_state.lock().await = PodState::Braking;
            if let Err(e) = self.tx_pod.send(255).await {
                eprintln!("trip->pod failed: {}", e);
            }

            sleep(Duration::from_secs_f32(time)).await;
            *self.pod_state.lock().await = PodState::Locked;
        }

        println!("trip_svc: service down");

        Ok(())
    }
}
