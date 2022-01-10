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
mod brake_svc;
use packet::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let (tx_auth_to_remote, rx_auth_to_remote) = mpsc::channel::<Packet>(32);
    let (tx_remote_to_auth, rx_remote_to_auth) = mpsc::channel::<Packet>(32);

    let (tx_auth_to_link, rx_auth_to_link) = mpsc::channel::<Packet>(32);
    let (tx_link_to_auth, rx_link_to_auth) = mpsc::channel::<Packet>(32);

    let (mut tx_brake_to_auth, mut rx_brake_to_auth) = mpsc::channel::<Packet>(32);
    let (mut tx_auth_to_brake, mut rx_auth_to_brake) = mpsc::channel::<Packet>(32);

    let (tx_brake_to_emerg, rx_brake_to_emerg) = mpsc::channel::<Packet>(32);
    let (tx_emerg_to_brake, rx_emerg_to_brake) = mpsc::channel::<Packet>(32);

    let (mut tx_brake_to_launch, mut rx_brake_to_launch) = mpsc::channel::<Packet>(32);
    let (tx_launch_to_break, mut rx_launch_to_break) = mpsc::channel::<Packet>(32);

    // create services with necessary control signals
    let auth_svc = auth_svc::AuthSvc { 
        rx_remote: rx_remote_to_auth,
        tx_remote: tx_auth_to_remote,

        rx_link: rx_link_to_auth,
        tx_link: tx_auth_to_link,

        rx_brake: rx_brake_to_auth,
        tx_brake: tx_auth_to_brake
    };

let brake_svc = brake_svc::BrakeSvc {
        rx_auth: rx_auth_to_brake,
        tx_auth: tx_brake_to_auth,

        rx_emerg: rx_emerg_to_brake, 
        tx_emerg: tx_brake_to_emerg,

        rx_launch: rx_launch_to_break,
        tx_launch: tx_brake_to_launch
    };

    let link_svc = link_svc::LinkSvc{ device_list: Vec::new(), rx: rx_auth_to_link, tx: tx_link_to_auth };
    let remote_conn_svc = remote_conn_svc::RemoteConnSvc { rx: rx_auth_to_remote, tx: tx_remote_to_auth };

    // spawn all services as tasks
    spawn(auth_svc.run());
    spawn(link_svc.run());
    spawn(remote_conn_svc.run());
    spawn(brake_svc.run());

    loop {

    }
}
