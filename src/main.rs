use anyhow::Result;
use std::sync::Arc;
use tokio::{spawn, sync::{mpsc, watch, Mutex}, task::spawn_blocking};

mod remote_conn_svc;
mod packet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let pod_state = Arc::new(Mutex::new(PodState::new()));
    // mpsc - multi producer, single consumer
    // feed commands from services to remote_conn_svc
    let (remote_in_tx, mut remote_in_rx) = mpsc::channel::<()>(32);
    // watch - single producer, mutli consumer
    // feed commands received from remote_conn_svc to corresponding services
    let (remote_out_tx, mut remote_out_rx) = watch::channel(());

    // spawn all services as tasks, and send corresponding control signals
    spawn(remote_conn_svc::server(remote_in_rx, remote_out_tx));

    loop {

    }
}

/*struct PodState {

}

impl PodState {
    fn new() -> Self {
        PodState {
            
        }
    }
}*/
