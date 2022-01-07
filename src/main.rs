use anyhow::Result;
use std::sync::Arc;
use tokio::{spawn, sync::{broadcast, mpsc, Mutex}};

mod remote_conn_svc;
mod packet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (remote_in_tx, remote_in_rx) = mpsc::channel::<i32>(32);
    let (remote_out_tx, remote_out_rx) = mpsc::channel::<i32>(32);

    let remote_conn_svc = remote_conn_svc::RemoteConnSvc { rx: remote_in_rx, tx: remote_out_tx };

    // spawn all services as tasks, and send corresponding control signals
    spawn(remote_conn_svc.run());

    loop {

    }
}
