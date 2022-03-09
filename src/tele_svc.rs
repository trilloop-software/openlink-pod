use rand::Rng;
use std::sync::Arc;
use tokio::{select, sync::{mpsc::Receiver, mpsc::Sender, Mutex}, time::{self, Duration}};

use shared::{remote_conn_packet::*, telemetry::*};
use super::pod_conn_svc::PodState;

pub struct TelemetrySvc {
    pub pod_state: Arc<Mutex<PodState>>,
    pub tele_data: Vec<TelemetryData>,

    pub rx_auth: Receiver<RemotePacket>,
    pub tx_auth: Sender<RemotePacket>,
    //pub rx_data: Receiver<u8>,
    //pub tx_data: Sender<u8>,
    //pub rx_emerg: Receiver<u8>,
    //pub tx_emerg: Sender<u8>,
    //pub rx_pod: Receiver<i32>,
    //pub tx_pod: Sender<i32>,
}

impl TelemetrySvc {
    pub async fn run(mut self) {
        println!("tele_svc: service running");
        self.init_fake_telemetry();

        // repeating interval to query subsystems for telemetry data
        let mut tele_timer = time::interval(Duration::from_secs(1));

        loop {
            select! {
                Some(pkt) = self.rx_auth.recv() => {
                    let resp = match pkt.cmd_type {
                        128 => self.report_telemetry().await,
                        _ => RemotePacket::new(0, vec![s!("Command not implemented")])
                    };

                    if let Err(e) = self.tx_auth.send(resp).await {
                        eprintln!("tele->auth failed: {}", e);
                    }
                }
                _ = tele_timer.tick() => self.get_telemetry().await
            }
        }
    }

    // TEMPORARY FUNCTION
    fn init_fake_telemetry(&mut self) {
        self.tele_data.push(TelemetryData::new(s!("Accelerometer"), 0.0, 120.0, 0.0));
        self.tele_data.push(TelemetryData::new(s!("Brake Temperature"), 0.0, 70.0, 30.0));
        self.tele_data.push(TelemetryData::new(s!("Battery Temperature"), 0.0, 50.0, 25.0));
        self.tele_data.push(TelemetryData::new(s!("Battery Current"), 0.0, 5.0, 0.0));
    }

    // TEMPORARY FUNCTION
    fn generate_fake_telemetry(&mut self) {
        let mut rng = rand::thread_rng();
        for el in &mut self.tele_data {
            el.field_value = rng.gen_range(el.value_lower..el.value_upper);
        }
    }

    async fn get_telemetry(&mut self) {
        let gather = match *self.pod_state.lock().await {
            PodState::Unlocked => false,
            _ => true
        };

        if gather {
            self.generate_fake_telemetry()
        }
    }

    // report both telemetry data and pod_state
    async fn report_telemetry(&mut self) -> RemotePacket {
        let pod_state = serde_json::to_string(&*self.pod_state.lock().await).unwrap();
        let telemetry = serde_json::to_string(&self.tele_data).unwrap();

        RemotePacket::new(128, vec![telemetry, pod_state])
    }
}
