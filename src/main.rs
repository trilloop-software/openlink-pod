use anyhow::Result;
use std::sync::Arc;
use tokio::{spawn, sync::{broadcast, mpsc, Mutex}};

#[macro_use]
mod macros;

mod device;
mod link_svc;
mod packet;
mod remote_conn_svc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (remote_in_tx, remote_in_rx) = mpsc::channel::<String>(32);
    let (remote_out_tx, remote_out_rx) = mpsc::channel::<String>(32);

    let link_svc = link_svc::LinkSvc{ device_list: Vec::new(), rx: remote_out_rx, tx: remote_in_tx };
    let remote_conn_svc = remote_conn_svc::RemoteConnSvc { rx: remote_in_rx, tx: remote_out_tx };

    // spawn all services as tasks, and send corresponding control signals
    spawn(link_svc.run());
    spawn(remote_conn_svc.run());

    loop {

    }
}
