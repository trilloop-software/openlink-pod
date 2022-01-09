use anyhow::Result;
use std::sync::Arc;
use tokio::{spawn, sync::{broadcast, mpsc, Mutex}};

#[macro_use]
mod macros;

mod auth_svc;
mod device;
mod link_svc;
mod packet;
mod remote_conn_svc;

use packet::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let (tx_auth_to_remote, rx_auth_to_remote) = mpsc::channel::<Packet>(32);
    let (tx_remote_to_auth, rx_remote_to_auth) = mpsc::channel::<Packet>(32);

    let (tx_auth_to_link, rx_auth_to_link) = mpsc::channel::<Packet>(32);
    let (tx_link_to_auth, rx_link_to_auth) = mpsc::channel::<Packet>(32);

    // create services with necessary control signals
    let auth_svc = auth_svc::AuthSvc { 
        rx_remote: rx_remote_to_auth,
        tx_remote: tx_auth_to_remote,
        rx_link: rx_link_to_auth,
        tx_link: tx_auth_to_link
    };
    let link_svc = link_svc::LinkSvc{ device_list: Vec::new(), rx: rx_auth_to_link, tx: tx_link_to_auth };
    let remote_conn_svc = remote_conn_svc::RemoteConnSvc { rx: rx_auth_to_remote, tx: tx_remote_to_auth };

    // spawn all services as tasks
    spawn(auth_svc.run());
    spawn(link_svc.run());
    spawn(remote_conn_svc.run());

    loop {

    }
}
