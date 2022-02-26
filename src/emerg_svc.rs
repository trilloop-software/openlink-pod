use anyhow::{Result};
use tokio::{sync::{mpsc::Receiver, mpsc::Sender}};

pub struct EmergSvc{
    pub rx_remote : Receiver<u8>,

    pub rx_pod : Receiver<u8>,
    pub tx_pod : Sender<u8>

    //pub rx_tele : Receiver<Packet>,
    //pub tx_tele : Sender<Packet>
}

impl EmergSvc {
    pub async fn run(mut self) -> Result<()> {
        println!("emerg_svc: service running");

        loop {
            tokio::select! {
                _ = self.rx_remote.recv() => {
                    // send to pod_conn_svc to engage breaks
                    // pod_conn_svc will engage breaks if PodState::Moving
                    // print line listing outcome Breaks Engaged, Not Moving
                    match self.tx_pod.send(1).await {
                        Ok(()) => {
                            let resp = self.rx_pod.recv().await;
                            match resp.unwrap() {
                                0 => println!("emerg_svc: braking unnecessary, pod_state != Moving"),
                                1 => println!("emerg_svc: brakes engaging"),
                                _ => println!("???")
                            }
                        },
                        Err(e) => eprintln!("emerg->pod failed: {}", e)
                    }
                }
                /*_ = self.rx_tele.recv() => {
                    // unsafe conditions met in telemetry_svc
                    // send to pod_conn_svc to engage breaks
                    // engage breaks regardless of state?
                }*/
            }
        }
    }
}